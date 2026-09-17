use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::db::migrate;
use crate::error::AppError;
use crate::models::{Client, ClientWrite, Dashboard, Note, RappelNtfy, Rdv, RdvDetail, Tarif};
use crate::overlap::overlaps;

pub fn jitsi_url(id: &str) -> String {
    format!("https://meet.jit.si/synapt-{id}")
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn ensure_migrated(conn: &Connection) -> Result<(), AppError> {
    migrate(conn)
}

pub(crate) const RDV_SELECT: &str = "\
SELECT rdv.id, rdv.client_id, rdv.tarif_id, rdv.debut, rdv.duree_minutes, \
rdv.jitsi_url, rdv.stripe_url, rdv.stripe_id, rdv.note, rdv.statut, \
rdv.created_at, rdv.updated_at, COALESCE(clients.nom, '') AS client_nom, \
COALESCE(tarifs.nom, '') AS tarif_nom \
FROM rdv \
LEFT JOIN clients ON clients.id = rdv.client_id \
LEFT JOIN tarifs ON tarifs.id = rdv.tarif_id";

const CLIENT_SELECT: &str = "SELECT id, nom, email, telephone, statut, memo, tarif_id,
    date_naissance, urgence_nom, urgence_telephone, orientation, frequence, adresse,
    created_at, updated_at FROM clients";

fn row_to_client(row: &Row<'_>) -> Result<Client, rusqlite::Error> {
    Ok(Client {
        id: row.get("id")?,
        nom: row.get("nom")?,
        email: row.get("email")?,
        telephone: row.get("telephone")?,
        statut: row.get("statut")?,
        memo: row.get("memo")?,
        tarif_id: row.get("tarif_id")?,
        date_naissance: row.get("date_naissance")?,
        urgence_nom: row.get("urgence_nom")?,
        urgence_telephone: row.get("urgence_telephone")?,
        orientation: row.get("orientation")?,
        frequence: row.get("frequence")?,
        adresse: row.get("adresse")?,
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
        prix_ttc: row.get::<_, i64>("prix_ttc")? != 0,
        actif: row.get::<_, i64>("actif")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub(crate) fn row_to_rdv(row: &Row<'_>) -> Result<Rdv, rusqlite::Error> {
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
        client_nom: row.get("client_nom")?,
        tarif_nom: row.get("tarif_nom")?,
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

pub fn list_rdvs_planifies(conn: &Connection) -> Result<Vec<Rdv>, AppError> {
    ensure_migrated(conn)?;
    let sql = format!("{RDV_SELECT} WHERE rdv.statut = 'planifie'");
    let mut stmt = conn.prepare(&sql)?;
    let rdvs = stmt
        .query_map([], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rdvs)
}

pub(crate) fn fetch_rdv(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    let sql = format!("{RDV_SELECT} WHERE rdv.id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let rdv = stmt.query_row(params![id], row_to_rdv)?;
    Ok(rdv)
}

pub fn parse_debut_utc(debut: &str) -> Result<DateTime<Utc>, AppError> {
    debut
        .parse::<DateTime<Utc>>()
        .map_err(|_| AppError::new("La date de début du rendez-vous n'est pas valide."))
}

pub fn assert_no_overlap(
    conn: &Connection,
    debut: &str,
    duree_minutes: i64,
    except_id: Option<&str>,
) -> Result<(), AppError> {
    parse_debut_utc(debut)?;
    let sql = if except_id.is_some() {
        "SELECT debut, duree_minutes FROM rdv WHERE statut = 'planifie' AND id != ?1"
    } else {
        "SELECT debut, duree_minutes FROM rdv WHERE statut = 'planifie'"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<(String, i64)> = if let Some(id) = except_id {
        stmt.query_map(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    };
    for (existing_debut, existing_duree) in rows {
        if overlaps(debut, duree_minutes, &existing_debut, existing_duree)? {
            return Err(AppError::new("chevauchement horaire"));
        }
    }
    Ok(())
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

fn rdv_debut_in_range(debut: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
    match parse_debut_utc(debut) {
        Ok(d) => d >= from && d < to,
        Err(_) => false,
    }
}

fn trim_opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn parse_statut(s: Option<String>) -> Result<String, AppError> {
    let v = trim_opt(s).unwrap_or_else(|| "en_cours".to_string());
    match v.as_str() {
        "en_cours" | "pause" | "termine" => Ok(v),
        _ => Err(AppError::new("Statut de suivi inconnu.")),
    }
}

fn parse_choice(
    value: Option<String>,
    allowed: &[&str],
    err: &str,
) -> Result<Option<String>, AppError> {
    let Some(v) = trim_opt(value) else {
        return Ok(None);
    };
    if allowed.contains(&v.as_str()) {
        Ok(Some(v))
    } else {
        Err(AppError::new(err))
    }
}

fn parse_date_naissance(value: Option<String>) -> Result<Option<String>, AppError> {
    let Some(v) = trim_opt(value) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(&v, "%Y-%m-%d")
        .map_err(|_| AppError::new("La date de naissance n'est pas valide."))?;
    Ok(Some(v))
}

pub fn clients_upsert(conn: &Connection, write: ClientWrite) -> Result<Client, AppError> {
    ensure_migrated(conn)?;
    let nom = write.nom.trim();
    if nom.is_empty() {
        return Err(AppError::new("Le nom est requis."));
    }
    let email = trim_opt(write.email);
    let telephone = trim_opt(write.telephone);
    let statut = parse_statut(write.statut)?;
    let memo = trim_opt(write.memo);
    if memo.as_ref().is_some_and(|m| m.chars().count() > 120) {
        return Err(AppError::new("Le mémo est trop long (120 caractères max)."));
    }
    let tarif_id = trim_opt(write.tarif_id);
    if let Some(tid) = tarif_id.as_deref() {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tarifs WHERE id = ?1",
            params![tid],
            |row| row.get(0),
        )?;
        if n == 0 {
            return Err(AppError::new("tarif introuvable"));
        }
    }
    let date_naissance = parse_date_naissance(write.date_naissance)?;
    let urgence_nom = trim_opt(write.urgence_nom);
    let urgence_telephone = trim_opt(write.urgence_telephone);
    let orientation = parse_choice(
        write.orientation,
        &["medecin", "reco", "lui_meme"],
        "Orientation inconnue.",
    )?;
    let frequence = parse_choice(
        write.frequence,
        &["hebdo", "bimensuel", "a_la_demande"],
        "Fréquence inconnue.",
    )?;
    let adresse = trim_opt(write.adresse);

    let now = now_iso();
    let id = match write.id {
        Some(existing) => existing,
        None => Uuid::new_v4().to_string(),
    };

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM clients WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if exists {
        conn.execute(
            "UPDATE clients SET nom = ?1, email = ?2, telephone = ?3, statut = ?4, memo = ?5,
             tarif_id = ?6, date_naissance = ?7, urgence_nom = ?8, urgence_telephone = ?9,
             orientation = ?10, frequence = ?11, adresse = ?12, updated_at = ?13 WHERE id = ?14",
            params![
                nom,
                email,
                telephone,
                statut,
                memo,
                tarif_id,
                date_naissance,
                urgence_nom,
                urgence_telephone,
                orientation,
                frequence,
                adresse,
                now,
                id
            ],
        )?;
    } else {
        conn.execute(
            "INSERT INTO clients (
               id, nom, email, telephone, statut, memo, tarif_id, date_naissance,
               urgence_nom, urgence_telephone, orientation, frequence, adresse, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
            params![
                id,
                nom,
                email,
                telephone,
                statut,
                memo,
                tarif_id,
                date_naissance,
                urgence_nom,
                urgence_telephone,
                orientation,
                frequence,
                adresse,
                now
            ],
        )?;
    }

    clients_get(conn, &id)
}

pub fn clients_list(conn: &Connection) -> Result<Vec<Client>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(&format!("{CLIENT_SELECT} ORDER BY nom"))?;
    let clients = stmt
        .query_map([], row_to_client)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(clients)
}

pub fn clients_get(conn: &Connection, id: &str) -> Result<Client, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(&format!("{CLIENT_SELECT} WHERE id = ?1"))?;
    let client = stmt.query_row(params![id], row_to_client)?;
    Ok(client)
}

pub fn clients_delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    ensure_migrated(conn)?;
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM clients WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::new("client introuvable"));
    }
    let planifies: i64 = conn.query_row(
        "SELECT COUNT(*) FROM rdv WHERE client_id = ?1 AND statut = 'planifie'",
        params![id],
        |row| row.get(0),
    )?;
    if planifies > 0 {
        return Err(AppError::new(
            "Ce client a encore des rendez-vous prévus. Annule-les avant de supprimer la fiche.",
        ));
    }
    let honoraires: i64 = conn.query_row(
        "SELECT COUNT(*) FROM honoraires WHERE client_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if honoraires > 0 {
        return Err(AppError::new("Ce client a encore des notes d'honoraires."));
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM rappels_ntfy WHERE rdv_id IN (SELECT id FROM rdv WHERE client_id = ?1)",
        params![id],
    )?;
    tx.execute("DELETE FROM rdv WHERE client_id = ?1", params![id])?;
    tx.execute("DELETE FROM notes WHERE client_id = ?1", params![id])?;
    tx.execute("DELETE FROM clients WHERE id = ?1", params![id])?;
    tx.commit()?;
    Ok(())
}

pub fn tarifs_upsert(
    conn: &Connection,
    id: Option<&str>,
    nom: &str,
    duree_minutes: i64,
    prix_centimes: i64,
    prix_ttc: bool,
) -> Result<Tarif, AppError> {
    ensure_migrated(conn)?;
    if nom.trim().is_empty() {
        return Err(AppError::new("Le nom du tarif est requis."));
    }
    if duree_minutes <= 0 {
        return Err(AppError::new("La durée doit être supérieure à zéro."));
    }
    if prix_centimes < 0 {
        return Err(AppError::new("Le prix ne peut pas être négatif."));
    }
    let nom = nom.trim();
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

    let prix_ttc_int = i64::from(prix_ttc);
    if exists {
        conn.execute(
        "UPDATE tarifs SET nom = ?1, duree_minutes = ?2, prix_centimes = ?3, prix_ttc = ?4, updated_at = ?5 WHERE id = ?6",
        params![nom, duree_minutes, prix_centimes, prix_ttc_int, now, id],
    )?;
    } else {
        conn.execute(
        "INSERT INTO tarifs (id, nom, duree_minutes, prix_centimes, prix_ttc, actif, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
        params![id, nom, duree_minutes, prix_centimes, prix_ttc_int, now],
    )?;
    }

    tarifs_get(conn, &id)
}

pub(crate) fn tarifs_get(conn: &Connection, id: &str) -> Result<Tarif, AppError> {
    let mut stmt = conn.prepare(
    "SELECT id, nom, duree_minutes, prix_centimes, prix_ttc, actif, created_at, updated_at FROM tarifs WHERE id = ?1",
)?;
    let tarif = stmt.query_row(params![id], row_to_tarif)?;
    Ok(tarif)
}

pub fn tarifs_list(conn: &Connection) -> Result<Vec<Tarif>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
    "SELECT id, nom, duree_minutes, prix_centimes, prix_ttc, actif, created_at, updated_at FROM tarifs ORDER BY nom",
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
    let mut stmt = conn
        .prepare("SELECT id, client_id, corps, created_at, updated_at FROM notes WHERE id = ?1")?;
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
    client_id: Option<&str>,
    tarif_id: Option<String>,
    debut: &str,
    duree_minutes: i64,
    note: Option<String>,
) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    if duree_minutes <= 0 {
        return Err(AppError::new("duree_minutes doit etre positif"));
    }

    let debut_norm = parse_debut_utc(debut)?.to_rfc3339();
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let insert = (|| {
        assert_no_overlap(conn, &debut_norm, duree_minutes, None)?;
        let id = Uuid::new_v4().to_string();
        let url = jitsi_url(&id);
        let now = now_iso();
        conn.execute(
            "INSERT INTO rdv (id, client_id, tarif_id, debut, duree_minutes, jitsi_url, stripe_url, stripe_id, note, statut, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, 'planifie', ?8, ?8)",
            params![
                id,
                client_id,
                tarif_id,
                debut_norm,
                duree_minutes,
                url,
                note,
                now
            ],
        )?;
        Ok(id)
    })();
    match insert {
        Ok(id) => {
            conn.execute_batch("COMMIT")?;
            fetch_rdv(conn, &id)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

pub fn rdv_update(
    conn: &Connection,
    id: &str,
    client_id: Option<&str>,
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

    let debut_norm = parse_debut_utc(debut)?.to_rfc3339();
    let tarif_changed = existing.tarif_id != tarif_id;
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let updated = (|| {
        assert_no_overlap(conn, &debut_norm, duree_minutes, Some(id))?;
        let now = now_iso();
        if tarif_changed {
            conn.execute(
                "UPDATE rdv SET client_id = ?1, tarif_id = ?2, debut = ?3, duree_minutes = ?4, note = ?5, stripe_url = NULL, stripe_id = NULL, updated_at = ?6 WHERE id = ?7",
                params![
                    client_id,
                    tarif_id,
                    debut_norm,
                    duree_minutes,
                    note,
                    now,
                    id
                ],
            )?;
        } else {
            conn.execute(
                "UPDATE rdv SET client_id = ?1, tarif_id = ?2, debut = ?3, duree_minutes = ?4, note = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    client_id,
                    tarif_id,
                    debut_norm,
                    duree_minutes,
                    note,
                    now,
                    id
                ],
            )?;
        }
        Ok(())
    })();
    match updated {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            fetch_rdv(conn, id)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

pub fn rdv_set_note(conn: &Connection, id: &str, note: Option<String>) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    let existing = fetch_rdv(conn, id)?;
    if existing.statut == "annule" {
        return Err(AppError::new("Ce rendez-vous est annulé."));
    }
    if existing.statut != "planifie" {
        return Err(AppError::new("rdv introuvable"));
    }
    let now = now_iso();
    conn.execute(
        "UPDATE rdv SET note = ?1, updated_at = ?2 WHERE id = ?3",
        params![note, now, id],
    )?;
    fetch_rdv(conn, id)
}

fn fetch_rappels(conn: &Connection, rdv_id: &str) -> Result<Vec<RappelNtfy>, AppError> {
    let mut stmt = conn.prepare(
    "SELECT id, rdv_id, type, ntfy_id, echeance, etat FROM rappels_ntfy WHERE rdv_id = ?1 ORDER BY echeance",
)?;
    let rappels = stmt
        .query_map(params![rdv_id], |row| {
            Ok(RappelNtfy {
                id: row.get(0)?,
                rdv_id: row.get(1)?,
                r#type: row.get(2)?,
                ntfy_id: row.get(3)?,
                echeance: row.get(4)?,
                etat: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rappels)
}

pub fn rdv_get(conn: &Connection, id: &str) -> Result<RdvDetail, AppError> {
    ensure_migrated(conn)?;
    let rdv = fetch_rdv(conn, id)?;
    let rappels = fetch_rappels(conn, id)?;
    Ok(RdvDetail { rdv, rappels })
}

pub fn rdv_annuler(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    ensure_migrated(conn)?;
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE rdv SET statut = 'annule', stripe_url = NULL, stripe_id = NULL, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    if updated == 0 {
        return Err(AppError::new("rdv introuvable"));
    }
    fetch_rdv(conn, id)
}

pub fn clients_rappels_ntfy_ids(
    conn: &Connection,
    client_id: &str,
) -> Result<Vec<String>, AppError> {
    ensure_migrated(conn)?;
    let mut stmt = conn.prepare(
        "SELECT ntfy_id FROM rappels_ntfy \
         WHERE rdv_id IN (SELECT id FROM rdv WHERE client_id = ?1) \
         AND etat = 'programme' AND ntfy_id IS NOT NULL",
    )?;
    let ids = stmt
        .query_map(params![client_id], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(ids)
}

pub fn rdv_list(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    client_id: Option<&str>,
) -> Result<Vec<Rdv>, AppError> {
    ensure_migrated(conn)?;
    if let Some(cid) = client_id {
        let sql = format!("{RDV_SELECT} WHERE rdv.client_id = ?1 ORDER BY rdv.debut");
        let mut stmt = conn.prepare(&sql)?;
        let rdvs = stmt
            .query_map(params![cid], row_to_rdv)?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(rdvs);
    }

    let from = from.ok_or_else(|| AppError::new("from requis"))?;
    let to = to.ok_or_else(|| AppError::new("to requis"))?;
    let from_dt = parse_debut_utc(from)?;
    let to_dt = parse_debut_utc(to)?;
    let sql = format!("{RDV_SELECT} WHERE rdv.statut = 'planifie' ORDER BY rdv.debut");
    let mut stmt = conn.prepare(&sql)?;
    let rdvs = stmt
        .query_map([], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;
    let rdvs = rdvs
        .into_iter()
        .filter(|r| rdv_debut_in_range(&r.debut, from_dt, to_dt))
        .collect();
    Ok(rdvs)
}

pub fn rdv_dashboard(conn: &Connection, now: DateTime<Utc>) -> Result<Dashboard, AppError> {
    ensure_migrated(conn)?;
    let (day_start, day_end) = local_day_bounds(now);
    let sql = format!("{RDV_SELECT} WHERE rdv.statut = 'planifie' ORDER BY rdv.debut");
    let mut stmt = conn.prepare(&sql)?;
    let all = stmt
        .query_map([], row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;

    let aujourdhui = all
        .iter()
        .filter(|r| rdv_debut_in_range(&r.debut, day_start, day_end))
        .cloned()
        .collect();

    let a_venir = all
        .into_iter()
        .filter(|r| {
            parse_debut_utc(&r.debut)
                .map(|d| d >= day_end)
                .unwrap_or(false)
        })
        .take(5)
        .collect();

    Ok(Dashboard {
        aujourdhui,
        a_venir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

    #[test]
    fn clients_upsert_roundtrip_adresse() {
        let conn = open_memory().unwrap();
        migrate(&conn).unwrap();
        let c = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Bob".into(),
                adresse: Some("1 rue A".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(c.adresse.as_deref(), Some("1 rue A"));
        let again = clients_get(&conn, &c.id).unwrap();
        assert_eq!(again.adresse.as_deref(), Some("1 rue A"));
    }

    #[test]
    fn clients_delete_refuse_si_honoraire() {
        use crate::honoraires::honoraires_create;
        use crate::settings::CabinetSettings;
        use std::path::PathBuf;

        let conn = open_memory().unwrap();
        migrate(&conn).unwrap();
        let client = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            None,
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let dir = PathBuf::from(std::env::temp_dir())
            .join(format!("synapt-hon-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("honoraires")).unwrap();
        let cabinet = CabinetSettings {
            nom: "Cabinet Test".into(),
            ..Default::default()
        };
        honoraires_create(&conn, &cabinet, &[rdv.id.clone()], "especes", &dir).unwrap();
        rdv_annuler(&conn, &rdv.id).unwrap();
        let err = clients_delete(&conn, &client.id).unwrap_err();
        assert_eq!(err.message, "Ce client a encore des notes d'honoraires.");
    }
}
