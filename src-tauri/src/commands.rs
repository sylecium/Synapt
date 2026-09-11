use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{db_path, migrate, open_file};
use crate::error::AppError;
use crate::models::{Client, Dashboard, Note, Rdv, Tarif};
use crate::ntfy::{echeance, should_publish, NtfyClient, RappelKind, ReqwestNtfy};
use crate::overlap::overlaps;
use crate::settings::{load_settings, save_settings, Settings, SettingsPublic};
use crate::stripe::stripe_create_payment_link;

#[derive(Debug, Clone, Serialize)]
pub struct RdvCreateResult {
    pub rdv: Rdv,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsSetInput {
    pub ntfy_serveur: String,
    pub ntfy_topic: String,
    pub ntfy_token: String,
    pub rappel_24h: bool,
    pub rappel_1h: bool,
    pub stripe_secret_key: String,
}

pub fn jitsi_url(id: &str) -> String {
    format!("https://meet.jit.si/synapt-{id}")
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn with_db<F, T>(f: F) -> Result<T, AppError>
where
    F: FnOnce(&Connection) -> Result<T, AppError>,
{
    let path = db_path()?;
    let conn = open_file(&path)?;
    migrate(&conn)?;
    f(&conn)
}

fn ensure_migrated(conn: &Connection) -> Result<(), AppError> {
    migrate(conn)
}

fn row_to_client(row: &Row<'_>) -> Result<Client, rusqlite::Error> {
    Ok(Client {
        id: row.get("id")?,
        nom: row.get("nom")?,
        email: row.get("email")?,
        telephone: row.get("telephone")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_tarif(row: &Row<'_>) -> Result<Tarif, rusqlite::Error> {
    Ok(Tarif {
        id: row.get("id")?,
        nom: row.get("nom")?,
        duree_minutes: row.get("duree_minutes")?,
        prix_centimes: row.get("prix_centimes")?,
        actif: row.get::<_, i64>("actif")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_rdv(row: &Row<'_>) -> Result<Rdv, rusqlite::Error> {
    Ok(Rdv {
        id: row.get("id")?,
        client_id: row.get("client_id")?,
        tarif_id: row.get("tarif_id")?,
        debut: row.get("debut")?,
        duree_minutes: row.get("duree_minutes")?,
        jitsi_url: row.get("jitsi_url")?,
        stripe_url: row.get("stripe_url")?,
        stripe_id: row.get("stripe_id")?,
        note: row.get("note")?,
        statut: row.get("statut")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_note(row: &Row<'_>) -> Result<Note, rusqlite::Error> {
    Ok(Note {
        id: row.get("id")?,
        client_id: row.get("client_id")?,
        corps: row.get("corps")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_rdv(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at FROM rdv WHERE id = ?1",
    )?;
    let rdv = stmt.query_row(params![id], row_to_rdv)?;
    Ok(rdv)
}

fn fetch_client_nom(conn: &Connection, client_id: &str) -> Result<String, AppError> {
    conn.query_row(
        "SELECT nom FROM clients WHERE id = ?1",
        params![client_id],
        |row| row.get(0),
    )
    .map_err(|_| AppError::new("client introuvable"))
}

fn fetch_tarif(conn: &Connection, tarif_id: &str) -> Result<Tarif, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, nom, duree_minutes, prix_centimes, actif, created_at, updated_at FROM tarifs WHERE id = ?1",
    )?;
    stmt.query_row(params![tarif_id], row_to_tarif)
        .map_err(|_| AppError::new("tarif introuvable"))
}

fn rappel_deja_programme(conn: &Connection, rdv_id: &str, kind: RappelKind) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM rappels_ntfy WHERE rdv_id = ?1 AND type = ?2 AND etat = 'programme'",
        params![rdv_id, kind.as_str()],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn insert_rappel(
    conn: &Connection,
    rdv_id: &str,
    kind: RappelKind,
    ntfy_id: &str,
    echeance_at: DateTime<Utc>,
) -> Result<(), AppError> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO rappels_ntfy (id, rdv_id, type, ntfy_id, echeance, etat) VALUES (?1, ?2, ?3, ?4, ?5, 'programme')",
        params![id, rdv_id, kind.as_str(), ntfy_id, echeance_at.to_rfc3339()],
    )?;
    Ok(())
}

fn kinds_for_settings(settings: &Settings) -> Vec<RappelKind> {
    let mut kinds = Vec::new();
    if settings.ntfy.rappel_24h {
        kinds.push(RappelKind::H24);
    }
    if settings.ntfy.rappel_1h {
        kinds.push(RappelKind::H1);
    }
    kinds
}

pub fn ntfy_sync<C: NtfyClient>(
    conn: &Connection,
    settings: &Settings,
    now: DateTime<Utc>,
    client_factory: impl Fn(&Rdv, RappelKind) -> C,
) -> Result<Vec<String>, AppError> {
    ensure_migrated(conn)?;
    let mut warnings = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at FROM rdv WHERE statut = 'planifie'",
    )?;
    let rdvs = stmt
        .query_map([], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;

    for rdv in rdvs {
        let debut: DateTime<Utc> = rdv.debut.parse()?;
        for kind in kinds_for_settings(settings) {
            let echeance_at = echeance(debut, kind);
            let deja = rappel_deja_programme(conn, &rdv.id, kind)?;
            if !should_publish(
                &settings.ntfy.topic,
                true,
                &rdv.statut,
                now,
                echeance_at,
                deja,
            ) {
                continue;
            }
            let ntfy_client = client_factory(&rdv, kind);
            match ntfy_client.publish(echeance_at, kind) {
                Ok(ntfy_id) => {
                    insert_rappel(conn, &rdv.id, kind, &ntfy_id, echeance_at)?;
                }
                Err(e) => {
                    eprintln!("ntfy_sync publish: {}", e.message);
                    warnings.push(e.message);
                }
            }
        }
    }
    Ok(warnings)
}

fn ntfy_schedule_rdv<C: NtfyClient>(
    conn: &Connection,
    settings: &Settings,
    rdv: &Rdv,
    now: DateTime<Utc>,
    client_factory: impl Fn(&Rdv, RappelKind) -> C,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let debut: DateTime<Utc> = match rdv.debut.parse() {
        Ok(d) => d,
        Err(e) => {
            warnings.push(e.to_string());
            return warnings;
        }
    };
    for kind in kinds_for_settings(settings) {
        let echeance_at = echeance(debut, kind);
        let deja = match rappel_deja_programme(conn, &rdv.id, kind) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(e.message);
                continue;
            }
        };
        if !should_publish(
            &settings.ntfy.topic,
            true,
            &rdv.statut,
            now,
            echeance_at,
            deja,
        ) {
            continue;
        }
        let ntfy_client = client_factory(rdv, kind);
        match ntfy_client.publish(echeance_at, kind) {
            Ok(ntfy_id) => {
                if let Err(e) = insert_rappel(conn, &rdv.id, kind, &ntfy_id, echeance_at) {
                    eprintln!("ntfy insert rappel: {}", e.message);
                    warnings.push(e.message);
                }
            }
            Err(e) => {
                eprintln!("ntfy publish: {}", e.message);
                warnings.push(e.message);
            }
        }
    }
    warnings
}

struct StripeTarifData {
    nom: String,
    prix_centimes: i64,
}

fn stripe_tarif_data(
    conn: &Connection,
    rdv: &Rdv,
    settings: &Settings,
) -> Result<Option<StripeTarifData>, AppError> {
    if settings.stripe.secret_key.is_empty() {
        return Ok(None);
    }
    if let Some(tarif_id) = &rdv.tarif_id {
        let tarif = fetch_tarif(conn, tarif_id)?;
        if tarif.prix_centimes > 0 {
            return Ok(Some(StripeTarifData {
                nom: tarif.nom,
                prix_centimes: tarif.prix_centimes,
            }));
        }
    }
    Ok(None)
}

fn save_stripe_link(
    conn: &Connection,
    rdv_id: &str,
    stripe_id: &str,
    stripe_url: &str,
) -> Result<Rdv, AppError> {
    let now = now_iso();
    conn.execute(
        "UPDATE rdv SET stripe_url = ?1, stripe_id = ?2, updated_at = ?3 WHERE id = ?4",
        params![stripe_url, stripe_id, now, rdv_id],
    )?;
    fetch_rdv(conn, rdv_id)
}

fn cancel_rappels_ntfy(conn: &Connection, settings: &Settings, rdv_id: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT ntfy_id FROM rappels_ntfy WHERE rdv_id = ?1 AND etat = 'programme' AND ntfy_id IS NOT NULL",
    ) {
        Ok(s) => s,
        Err(e) => {
            warnings.push(e.to_string());
            return warnings;
        }
    };
    let ntfy_ids: Vec<String> = match stmt.query_map(params![rdv_id], |row| row.get(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            warnings.push(e.to_string());
            Vec::new()
        }
    };

    for ntfy_id in ntfy_ids {
        if let Err(e) = tauri::async_runtime::block_on(crate::ntfy::ntfy_delete(settings, &ntfy_id))
        {
            eprintln!("ntfy delete: {}", e.message);
            warnings.push(e.message);
        }
    }

    if let Err(e) = conn.execute(
        "UPDATE rappels_ntfy SET etat = 'annule' WHERE rdv_id = ?1 AND etat = 'programme'",
        params![rdv_id],
    ) {
        warnings.push(e.to_string());
    }
    warnings
}

fn local_day_bounds(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let local = now.with_timezone(&Local);
    let day_start = local.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let day_end = day_start + Duration::days(1);
    let start_utc = Local
        .from_local_datetime(&day_start)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let end_utc = Local
        .from_local_datetime(&day_end)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    (start_utc, end_utc)
}

pub mod repo {
    use super::*;

    pub fn clients_upsert(
    conn: &Connection,
    id: Option<&str>,
    nom: &str,
    email: Option<&str>,
    telephone: Option<&str>,
) -> Result<Client, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let id = match id {
        Some(existing) => existing.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM clients WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if exists {
        conn.execute(
            "UPDATE clients SET nom = ?1, email = ?2, telephone = ?3, updated_at = ?4 WHERE id = ?5",
            params![nom, email, telephone, now, id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO clients (id, nom, email, telephone, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![id, nom, email, telephone, now],
        )?;
    }

    clients_get(conn, &id)
}

pub fn clients_list(conn: &Connection) -> Result<Vec<Client>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, nom, email, telephone, created_at, updated_at FROM clients ORDER BY nom",
    )?;
    let clients = stmt
        .query_map([], row_to_client)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(clients)
}

pub fn clients_get(conn: &Connection, id: &str) -> Result<Client, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, nom, email, telephone, created_at, updated_at FROM clients WHERE id = ?1",
    )?;
    let client = stmt.query_row(params![id], row_to_client)?;
    Ok(client)
}

pub fn tarifs_upsert(
    conn: &Connection,
    id: Option<&str>,
    nom: &str,
    duree_minutes: i64,
    prix_centimes: i64,
) -> Result<Tarif, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let id = match id {
        Some(existing) => existing.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM tarifs WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if exists {
        conn.execute(
            "UPDATE tarifs SET nom = ?1, duree_minutes = ?2, prix_centimes = ?3, updated_at = ?4 WHERE id = ?5",
            params![nom, duree_minutes, prix_centimes, now, id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO tarifs (id, nom, duree_minutes, prix_centimes, actif, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
            params![id, nom, duree_minutes, prix_centimes, now],
        )?;
    }

    tarifs_get(conn, &id)
}

fn tarifs_get(conn: &Connection, id: &str) -> Result<Tarif, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, nom, duree_minutes, prix_centimes, actif, created_at, updated_at FROM tarifs WHERE id = ?1",
    )?;
    let tarif = stmt.query_row(params![id], row_to_tarif)?;
    Ok(tarif)
}

pub fn tarifs_list(conn: &Connection) -> Result<Vec<Tarif>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, nom, duree_minutes, prix_centimes, actif, created_at, updated_at FROM tarifs ORDER BY nom",
    )?;
    let tarifs = stmt
        .query_map([], row_to_tarif)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tarifs)
}

pub fn tarifs_set_actif(conn: &Connection, id: &str, actif: bool) -> Result<Tarif, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let actif_int = i64::from(actif);
    let updated = conn.execute(
        "UPDATE tarifs SET actif = ?1, updated_at = ?2 WHERE id = ?3",
        params![actif_int, now, id],
    )?;
    if updated == 0 {
        return Err(AppError::new("tarif introuvable"));
    }
    tarifs_get(conn, id)
}

pub fn notes_upsert(
    conn: &Connection,
    id: Option<&str>,
    client_id: Option<&str>,
    corps: &str,
) -> Result<Note, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let id = match id {
        Some(existing) => existing.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if exists {
        conn.execute(
            "UPDATE notes SET client_id = ?1, corps = ?2, updated_at = ?3 WHERE id = ?4",
            params![client_id, corps, now, id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO notes (id, client_id, corps, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, client_id, corps, now],
        )?;
    }

    notes_get(conn, &id)
}

fn notes_get(conn: &Connection, id: &str) -> Result<Note, AppError> {
    let mut stmt =
        conn.prepare("SELECT id, client_id, corps, created_at, updated_at FROM notes WHERE id = ?1")?;
    let note = stmt.query_row(params![id], row_to_note)?;
    Ok(note)
}

pub fn notes_list(
    conn: &Connection,
    client_id: Option<String>,
    perso: bool,
) -> Result<Vec<Note>, AppError> {
    ensure_migrated(conn)?;
    if perso {
        let mut stmt = conn.prepare(
            "SELECT id, client_id, corps, created_at, updated_at FROM notes WHERE client_id IS NULL ORDER BY updated_at DESC",
        )?;
        return stmt
            .query_map([], row_to_note)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from);
    }
    if let Some(client_id) = client_id {
        let mut stmt = conn.prepare(
            "SELECT id, client_id, corps, created_at, updated_at FROM notes WHERE client_id = ?1 ORDER BY updated_at DESC",
        )?;
        return stmt
            .query_map(params![client_id], row_to_note)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from);
    }
    let mut stmt = conn.prepare(
        "SELECT id, client_id, corps, created_at, updated_at FROM notes ORDER BY updated_at DESC",
    )?;
    let notes = stmt
        .query_map([], row_to_note)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

pub fn notes_delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    ensure_migrated(conn)?;
    let deleted = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    if deleted == 0 {
        return Err(AppError::new("note introuvable"));
    }
    Ok(())
}

pub fn rdv_create(
    conn: &Connection,
    client_id: &str,
    tarif_id: Option<String>,
    debut: &str,
    duree_minutes: i64,
    note: Option<String>,
) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    if duree_minutes <= 0 {
        return Err(AppError::new("duree_minutes doit etre positif"));
    }

    let mut stmt = conn.prepare(
        "SELECT debut, duree_minutes FROM rdv WHERE statut = 'planifie'",
    )?;
    let existing = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (existing_debut, existing_duree) in existing {
        if overlaps(debut, duree_minutes, &existing_debut, existing_duree)? {
            return Err(AppError::new("chevauchement horaire"));
        }
    }

    let id = Uuid::new_v4().to_string();
    let url = jitsi_url(&id);
    let now = now_iso();
    conn.execute(
        "INSERT INTO rdv (id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, 'planifie', ?8, ?8)",
        params![id, client_id, tarif_id, debut, duree_minutes, url, note, now],
    )?;

    fetch_rdv(conn, &id)
}

pub fn rdv_update(
    conn: &Connection,
    id: &str,
    client_id: &str,
    tarif_id: Option<String>,
    debut: &str,
    duree_minutes: i64,
    note: Option<String>,
) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    if duree_minutes <= 0 {
        return Err(AppError::new("duree_minutes doit etre positif"));
    }

    let existing = fetch_rdv(conn, id)?;
    if existing.statut != "planifie" {
        return Err(AppError::new("rdv non modifiable"));
    }

    let mut stmt = conn.prepare(
        "SELECT id, debut, duree_minutes FROM rdv WHERE statut = 'planifie' AND id != ?1",
    )?;
    let others = stmt
        .query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (_, other_debut, other_duree) in others {
        if overlaps(debut, duree_minutes, &other_debut, other_duree)? {
            return Err(AppError::new("chevauchement horaire"));
        }
    }

    let now = now_iso();
    conn.execute(
        "UPDATE rdv SET client_id = ?1, tarif_id = ?2, debut = ?3, duree_minutes = ?4, note = ?5, updated_at = ?6 WHERE id = ?7",
        params![client_id, tarif_id, debut, duree_minutes, note, now, id],
    )?;

    fetch_rdv(conn, id)
}

pub fn rdv_get(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    fetch_rdv(conn, id)
}

pub fn rdv_annuler(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE rdv SET statut = 'annule', updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    if updated == 0 {
        return Err(AppError::new("rdv introuvable"));
    }
    fetch_rdv(conn, id)
}

pub fn rdv_list(conn: &Connection, from: &str, to: &str) -> Result<Vec<Rdv>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at FROM rdv WHERE debut >= ?1 AND debut < ?2 AND statut = 'planifie' ORDER BY debut",
    )?;
    let rdvs = stmt
        .query_map(params![from, to], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rdvs)
}

pub fn rdv_dashboard(conn: &Connection, now: DateTime<Utc>) -> Result<Dashboard, AppError> {
    ensure_migrated(conn)?;
    let (day_start, day_end) = local_day_bounds(now);
    let aujourdhui = rdv_list(conn, &day_start.to_rfc3339(), &day_end.to_rfc3339())?;

    let mut stmt = conn.prepare(
        "SELECT id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at FROM rdv WHERE debut >= ?1 AND statut = 'planifie' ORDER BY debut LIMIT 5",
    )?;
    let a_venir = stmt
        .query_map(params![day_end.to_rfc3339()], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Dashboard { aujourdhui, a_venir })
    }
}

pub fn settings_apply(input: &SettingsSetInput, current: &Settings) -> Settings {
    Settings {
        ntfy: crate::settings::NtfySettings {
            serveur: input.ntfy_serveur.clone(),
            topic: input.ntfy_topic.clone(),
            token: if input.ntfy_token.is_empty() {
                current.ntfy.token.clone()
            } else {
                input.ntfy_token.clone()
            },
            rappel_24h: input.rappel_24h,
            rappel_1h: input.rappel_1h,
        },
        stripe: crate::settings::StripeSettings {
            secret_key: if input.stripe_secret_key.is_empty() {
                current.stripe.secret_key.clone()
            } else {
                input.stripe_secret_key.clone()
            },
        },
    }
}

// --- Tauri commands ---

#[tauri::command]
pub fn settings_get() -> SettingsPublic {
    load_settings().to_public()
}

#[tauri::command]
pub fn settings_set(input: SettingsSetInput) -> Result<(), String> {
    let current = load_settings();
    let updated = settings_apply(&input, &current);
    save_settings(&updated).map_err(|e| e.message)
}

#[tauri::command]
pub fn clients_list() -> Result<Vec<Client>, String> {
    with_db(repo::clients_list).map_err(|e| e.message)
}

#[tauri::command]
pub fn clients_get(id: String) -> Result<Client, String> {
    with_db(|conn| repo::clients_get(conn, &id)).map_err(|e| e.message)
}

#[tauri::command]
pub fn clients_upsert(
    id: Option<String>,
    nom: String,
    email: Option<String>,
    telephone: Option<String>,
) -> Result<Client, String> {
    with_db(|conn| {
        repo::clients_upsert(
            conn,
            id.as_deref(),
            &nom,
            email.as_deref(),
            telephone.as_deref(),
        )
    })
    .map_err(|e| e.message)
}

#[tauri::command]
pub fn tarifs_list() -> Result<Vec<Tarif>, String> {
    with_db(repo::tarifs_list).map_err(|e| e.message)
}

#[tauri::command]
pub fn tarifs_upsert(
    id: Option<String>,
    nom: String,
    duree_minutes: i64,
    prix_centimes: i64,
) -> Result<Tarif, String> {
    with_db(|conn| repo::tarifs_upsert(conn, id.as_deref(), &nom, duree_minutes, prix_centimes))
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn tarifs_set_actif(id: String, actif: bool) -> Result<Tarif, String> {
    with_db(|conn| repo::tarifs_set_actif(conn, &id, actif)).map_err(|e| e.message)
}

#[tauri::command]
pub fn notes_list(client_id: Option<String>, perso: bool) -> Result<Vec<Note>, String> {
    with_db(|conn| repo::notes_list(conn, client_id, perso)).map_err(|e| e.message)
}

#[tauri::command]
pub fn notes_upsert(
    id: Option<String>,
    client_id: Option<String>,
    corps: String,
) -> Result<Note, String> {
    with_db(|conn| {
        repo::notes_upsert(conn, id.as_deref(), client_id.as_deref(), &corps)
    })
    .map_err(|e| e.message)
}

#[tauri::command]
pub fn notes_delete(id: String) -> Result<(), String> {
    with_db(|conn| repo::notes_delete(conn, &id)).map_err(|e| e.message)
}

#[tauri::command]
pub fn rdv_list(from: String, to: String) -> Result<Vec<Rdv>, String> {
    with_db(|conn| repo::rdv_list(conn, &from, &to)).map_err(|e| e.message)
}

#[tauri::command]
pub fn rdv_get(id: String) -> Result<Rdv, String> {
    with_db(|conn| repo::rdv_get(conn, &id)).map_err(|e| e.message)
}

#[tauri::command]
pub async fn rdv_create(
    client_id: String,
    tarif_id: Option<String>,
    debut: String,
    duree_minutes: i64,
    note: Option<String>,
) -> Result<RdvCreateResult, String> {
    let settings = load_settings();
    let now = Utc::now();

    let rdv = with_db(|conn| {
        repo::rdv_create(
            conn,
            &client_id,
            tarif_id.clone(),
            &debut,
            duree_minutes,
            note.clone(),
        )
    })
    .map_err(|e| e.message)?;

    let tarif_data =
        with_db(|conn| stripe_tarif_data(conn, &rdv, &settings)).map_err(|e| e.message)?;

    let mut warnings = Vec::new();
    let rdv = if let Some(data) = tarif_data {
        match stripe_create_payment_link(
            &settings.stripe.secret_key,
            &data.nom,
            data.prix_centimes,
        )
        .await
        {
            Ok((stripe_id, stripe_url)) => with_db(|conn| save_stripe_link(conn, &rdv.id, &stripe_id, &stripe_url))
                .map_err(|e| e.message)?,
            Err(e) => {
                eprintln!("stripe: {}", e.message);
                warnings.push(e.message);
                rdv
            }
        }
    } else {
        rdv
    };

    let ntfy_warnings = with_db(|conn| {
        Ok(ntfy_schedule_rdv(conn, &settings, &rdv, now, |r, kind| {
            let client_nom = fetch_client_nom(conn, &r.client_id).unwrap_or_default();
            let debut_dt = r.debut.parse().unwrap_or(now);
            ReqwestNtfy::for_rdv(&settings, &client_nom, debut_dt, &r.jitsi_url, kind)
        }))
    })
    .map_err(|e| e.message)?;
    warnings.extend(ntfy_warnings);

    Ok(RdvCreateResult { rdv, warnings })
}

#[tauri::command]
pub async fn rdv_update(
    id: String,
    client_id: String,
    tarif_id: Option<String>,
    debut: String,
    duree_minutes: i64,
    note: Option<String>,
) -> Result<Rdv, String> {
    let settings = load_settings();
    let now = Utc::now();

    let existing = with_db(|conn| repo::rdv_get(conn, &id)).map_err(|e| e.message)?;
    let debut_changed = existing.debut != debut;

    if debut_changed {
        let _ = with_db(|conn| {
            let ws = cancel_rappels_ntfy(conn, &settings, &id);
            if !ws.is_empty() {
                for w in ws {
                    eprintln!("rdv_update ntfy cancel: {}", w);
                }
            }
            Ok(())
        });
    }

    let rdv = with_db(|conn| {
        repo::rdv_update(
            conn,
            &id,
            &client_id,
            tarif_id.clone(),
            &debut,
            duree_minutes,
            note.clone(),
        )
    })
    .map_err(|e| e.message)?;

    if debut_changed {
        let _ = with_db(|conn| {
            let ws = ntfy_schedule_rdv(conn, &settings, &rdv, now, |r, kind| {
                let client_nom = fetch_client_nom(conn, &r.client_id).unwrap_or_default();
                let debut_dt = r.debut.parse().unwrap_or(now);
                ReqwestNtfy::for_rdv(&settings, &client_nom, debut_dt, &r.jitsi_url, kind)
            });
            for w in ws {
                eprintln!("rdv_update ntfy schedule: {}", w);
            }
            Ok(())
        });
    }

    Ok(rdv)
}

#[tauri::command]
pub async fn rdv_annuler(id: String) -> Result<Rdv, String> {
    let settings = load_settings();
    let warnings = with_db(|conn| Ok(cancel_rappels_ntfy(conn, &settings, &id)))
        .map_err(|e| e.message)?;
    for w in warnings {
        eprintln!("rdv_annuler ntfy: {}", w);
    }
    with_db(|conn| repo::rdv_annuler(conn, &id)).map_err(|e| e.message)
}

#[tauri::command]
pub fn rdv_dashboard() -> Result<Dashboard, String> {
    with_db(|conn| repo::rdv_dashboard(conn, Utc::now())).map_err(|e| e.message)
}

#[tauri::command]
pub async fn stripe_ensure_link(rdv_id: String) -> Result<Rdv, String> {
    let settings = load_settings();
    let rdv = with_db(|conn| fetch_rdv(conn, &rdv_id)).map_err(|e| e.message)?;
    if rdv.stripe_url.is_some() {
        return Ok(rdv);
    }
    let tarif_data =
        with_db(|conn| stripe_tarif_data(conn, &rdv, &settings)).map_err(|e| e.message)?;
    if let Some(data) = tarif_data {
        match stripe_create_payment_link(
            &settings.stripe.secret_key,
            &data.nom,
            data.prix_centimes,
        )
        .await
        {
            Ok((stripe_id, stripe_url)) => {
                return with_db(|conn| save_stripe_link(conn, &rdv.id, &stripe_id, &stripe_url))
                    .map_err(|e| e.message);
            }
            Err(e) => return Err(e.message),
        }
    }
    Ok(rdv)
}

#[tauri::command]
pub async fn ntfy_test() -> Result<(), String> {
    let settings = load_settings();
    crate::ntfy::ntfy_test(&settings)
        .await
        .map_err(|e| e.message)
}

pub fn run_ntfy_sync() {
    let settings = load_settings();
    let now = Utc::now();
    if let Ok(path) = db_path() {
        if let Ok(conn) = open_file(&path) {
            if let Err(e) = migrate(&conn) {
                eprintln!("ntfy_sync migrate: {}", e.message);
                return;
            }
            let result = ntfy_sync(&conn, &settings, now, |rdv, kind| {
                let client_nom = fetch_client_nom(&conn, &rdv.client_id).unwrap_or_default();
                let debut_dt = rdv.debut.parse().unwrap_or(now);
                ReqwestNtfy::for_rdv(&settings, &client_nom, debut_dt, &rdv.jitsi_url, kind)
            });
            if let Ok(warnings) = result {
                for w in warnings {
                    eprintln!("ntfy_sync: {}", w);
                }
            } else if let Err(e) = result {
                eprintln!("ntfy_sync: {}", e.message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::repo::*;
    use super::{ntfy_sync, settings_apply, SettingsSetInput};
    use crate::db::migrate;
    use crate::error::AppError;
    use crate::models::{Client, Tarif};
    use crate::ntfy::{NtfyClient, RappelKind};
    use crate::settings::Settings;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use rusqlite::Connection;
    use std::sync::Mutex;

    struct FakeNtfy {
        published: std::sync::Arc<Mutex<Vec<RappelKind>>>,
    }

    impl Clone for FakeNtfy {
        fn clone(&self) -> Self {
            Self {
                published: std::sync::Arc::clone(&self.published),
            }
        }
    }

    impl NtfyClient for FakeNtfy {
        fn publish(&self, _delay: DateTime<Utc>, kind: RappelKind) -> Result<String, AppError> {
            self.published.lock().unwrap().push(kind);
            Ok("fake-ntfy-id".to_string())
        }
    }

    fn seed_client(conn: &Connection) -> Client {
        migrate(conn).unwrap();
        clients_upsert(conn, None, "Alice", None, None).unwrap()
    }

    fn seed_tarif(conn: &Connection) -> Tarif {
        migrate(conn).unwrap();
        tarifs_upsert(conn, None, "Consultation", 60, 5000).unwrap()
    }

    fn test_settings(topic: &str, rappel_1h: bool) -> Settings {
        Settings {
            ntfy: crate::settings::NtfySettings {
                serveur: "https://ntfy.sh".to_string(),
                topic: topic.to_string(),
                token: String::new(),
                rappel_24h: false,
                rappel_1h,
            },
            stripe: crate::settings::StripeSettings {
                secret_key: String::new(),
            },
        }
    }

    #[test]
    fn create_rdv_jitsi_et_planifie() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let tarif = seed_tarif(&conn);
        let rdv = rdv_create(
            &conn,
            &client.id,
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert!(rdv.jitsi_url.starts_with("https://meet.jit.si/synapt-"));
        assert_eq!(rdv.statut, "planifie");
    }

    #[test]
    fn conflit_horaire_refuse() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        let t = seed_tarif(&conn);
        rdv_create(
            &conn,
            &c.id,
            Some(t.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let err = rdv_create(
            &conn,
            &c.id,
            Some(t.id),
            "2026-09-11T10:30:00Z",
            30,
            None,
        )
        .unwrap_err();
        assert!(
            err.message.contains("chevauche") || err.message.contains("horaire"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn annuler_libere_le_creneau() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        let t = seed_tarif(&conn);
        let a = rdv_create(
            &conn,
            &c.id,
            Some(t.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_annuler(&conn, &a.id).unwrap();
        let b = rdv_create(
            &conn,
            &c.id,
            Some(t.id),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert_eq!(b.statut, "planifie");
    }

    #[test]
    fn ntfy_sync_10_jours_ne_publie_pas() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let now = Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).unwrap();
        let debut = (now + Duration::days(10)).to_rfc3339();
        rdv_create(&conn, &client.id, None, &debut, 60, None).unwrap();
        let settings = test_settings("topic-test", true);
        let fake = FakeNtfy {
            published: std::sync::Arc::new(Mutex::new(vec![])),
        };
        ntfy_sync(&conn, &settings, now, |_rdv, _kind| fake.clone()).unwrap();
        assert!(fake.published.lock().unwrap().is_empty());
    }

    #[test]
    fn ntfy_sync_2h_publie_h1() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let now = Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).unwrap();
        let debut = (now + Duration::hours(2)).to_rfc3339();
        rdv_create(&conn, &client.id, None, &debut, 60, None).unwrap();
        let settings = test_settings("topic-test", true);
        let fake = FakeNtfy {
            published: std::sync::Arc::new(Mutex::new(vec![])),
        };
        ntfy_sync(&conn, &settings, now, |_rdv, _kind| fake.clone()).unwrap();
        let published = fake.published.lock().unwrap().clone();
        assert_eq!(published, vec![RappelKind::H1]);
    }

    #[test]
    fn settings_set_garde_secrets_vides() {
        let current = Settings {
            ntfy: crate::settings::NtfySettings {
                serveur: "https://ntfy.sh".to_string(),
                topic: "t".to_string(),
                token: "tok_secret".to_string(),
                rappel_24h: true,
                rappel_1h: true,
            },
            stripe: crate::settings::StripeSettings {
                secret_key: "sk_test_secret".to_string(),
            },
        };
        let input = SettingsSetInput {
            ntfy_serveur: "https://ntfy.sh".to_string(),
            ntfy_topic: "new-topic".to_string(),
            ntfy_token: String::new(),
            rappel_24h: false,
            rappel_1h: true,
            stripe_secret_key: String::new(),
        };
        let updated = settings_apply(&input, &current);
        assert_eq!(updated.ntfy.token, "tok_secret");
        assert_eq!(updated.stripe.secret_key, "sk_test_secret");
        assert_eq!(updated.ntfy.topic, "new-topic");
        assert!(!updated.ntfy.rappel_24h);
    }
}
