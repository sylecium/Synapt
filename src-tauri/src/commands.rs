use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use uuid::Uuid;

use crate::db::DbState;
use crate::error::AppError;
use crate::models::{
    Client, ClientWrite, Dashboard, Honoraire, HonoraireDetail, Note, Rdv, RdvDetail, Tarif,
};
use crate::ntfy::{echeance, ntfy_delete, should_publish, NtfyClient, RappelKind, ReqwestNtfy};
use crate::repo::{self, fetch_rdv, list_rdvs_planifies, now_iso};
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
    pub ntfy_token_clear: bool,
    pub stripe_secret_clear: bool,
    pub cabinet_nom: String,
    pub cabinet_adresse: String,
    pub cabinet_telephone: String,
    pub cabinet_email: String,
    pub cabinet_siret: String,
    pub mention_tva: String,
    pub prefixe_numero: String,
}

fn fetch_tarif(conn: &Connection, tarif_id: &str) -> Result<Tarif, AppError> {
    repo::tarifs_get(conn, tarif_id).map_err(|_| AppError::new("tarif introuvable"))
}

fn rappel_deja_programme(
    conn: &Connection,
    rdv_id: &str,
    kind: RappelKind,
) -> Result<bool, AppError> {
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
        "INSERT INTO rappels_ntfy (id, rdv_id, type, ntfy_id, echeance, etat) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'programme') \
         ON CONFLICT(rdv_id, type) DO UPDATE SET \
         etat = 'programme', ntfy_id = excluded.ntfy_id, echeance = excluded.echeance",
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

struct PendingNtfyJob {
    rdv_id: String,
    kind: RappelKind,
    echeance_at: DateTime<Utc>,
    client_nom: String,
    debut: DateTime<Utc>,
    jitsi_url: String,
    replace_ntfy_id: Option<String>,
}

fn existing_rappel_ntfy_id(
    conn: &Connection,
    rdv_id: &str,
    kind: RappelKind,
) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT ntfy_id FROM rappels_ntfy WHERE rdv_id = ?1 AND type = ?2",
        params![rdv_id, kind.as_str()],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map_err(AppError::from)
    .map(|opt| opt.flatten())
}

fn ntfy_collect_pending(
    conn: &Connection,
    settings: &Settings,
    now: DateTime<Utc>,
    only_rdv_id: Option<&str>,
) -> Result<Vec<PendingNtfyJob>, AppError> {
    let rdvs = list_rdvs_planifies(conn)?;
    let mut jobs = Vec::new();
    for rdv in rdvs {
        if only_rdv_id.is_some_and(|id| id != rdv.id) {
            continue;
        }
        let debut = repo::parse_debut_utc(&rdv.debut)?;
        let client_nom = rdv.client_nom.clone();
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
            let replace_ntfy_id = existing_rappel_ntfy_id(conn, &rdv.id, kind)?;
            jobs.push(PendingNtfyJob {
                rdv_id: rdv.id.clone(),
                kind,
                echeance_at,
                client_nom: client_nom.clone(),
                debut,
                jitsi_url: rdv.jitsi_url.clone(),
                replace_ntfy_id,
            });
        }
    }
    Ok(jobs)
}

pub fn ntfy_sync<C: NtfyClient>(
    conn: &Connection,
    settings: &Settings,
    now: DateTime<Utc>,
    client_factory: impl Fn(&Rdv, RappelKind) -> C,
) -> Result<Vec<String>, AppError> {
    let jobs = ntfy_collect_pending(conn, settings, now, None)?;
    let mut warnings = Vec::new();
    let rdvs = list_rdvs_planifies(conn)?;
    for job in jobs {
        let rdv = rdvs
            .iter()
            .find(|r| r.id == job.rdv_id)
            .ok_or_else(|| AppError::new("rdv introuvable"))?;
        let ntfy_client = client_factory(rdv, job.kind);
        match ntfy_client.publish(job.echeance_at, job.kind) {
            Ok(ntfy_id) => {
                if let Err(e) =
                    insert_rappel(conn, &job.rdv_id, job.kind, &ntfy_id, job.echeance_at)
                {
                    warnings.push(e.message);
                }
            }
            Err(e) => {
                eprintln!("ntfy_sync publish: {}", e.message);
                warnings.push(e.message);
            }
        }
    }
    Ok(warnings)
}

async fn ntfy_run_pending_jobs(
    db: &DbState,
    settings: &Settings,
    jobs: Vec<PendingNtfyJob>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for job in jobs {
        let ntfy = ReqwestNtfy::for_rdv(
            settings,
            &job.client_nom,
            job.debut,
            &job.jitsi_url,
            job.kind,
        );
        match ntfy.publish_async(job.echeance_at).await {
            Ok(ntfy_id) => {
                if let Some(old) = job.replace_ntfy_id {
                    if old != ntfy_id {
                        if let Err(e) = ntfy_delete(settings, &old).await {
                            eprintln!("ntfy delete old: {}", e.message);
                            warnings.push(e.message);
                        }
                    }
                }
                if let Err(e) = db.with_conn(|conn| {
                    insert_rappel(conn, &job.rdv_id, job.kind, &ntfy_id, job.echeance_at)
                }) {
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

async fn ntfy_schedule_rdv(
    db: &DbState,
    settings: &Settings,
    rdv_id: &str,
    now: DateTime<Utc>,
) -> Vec<String> {
    let jobs = match db.with_conn(|conn| ntfy_collect_pending(conn, settings, now, Some(rdv_id))) {
        Ok(j) => j,
        Err(e) => return vec![e.message],
    };
    ntfy_run_pending_jobs(db, settings, jobs).await
}

fn rappels_ntfy_ids(conn: &Connection, rdv_id: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT ntfy_id FROM rappels_ntfy WHERE rdv_id = ?1 AND etat = 'programme' AND ntfy_id IS NOT NULL",
    )?;
    let ids = stmt
        .query_map(params![rdv_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(ids)
}

fn mark_rappels_annule(conn: &Connection, rdv_id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE rappels_ntfy SET etat = 'annule' WHERE rdv_id = ?1 AND etat = 'programme'",
        params![rdv_id],
    )?;
    Ok(())
}

async fn cancel_rappels_ntfy(db: &DbState, settings: &Settings, rdv_id: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let ntfy_ids = match db.with_conn(|conn| rappels_ntfy_ids(conn, rdv_id)) {
        Ok(ids) => ids,
        Err(e) => {
            warnings.push(e.message);
            return warnings;
        }
    };

    for ntfy_id in ntfy_ids {
        if let Err(e) = ntfy_delete(settings, &ntfy_id).await {
            eprintln!("ntfy delete: {}", e.message);
            warnings.push(e.message);
        }
    }

    if let Err(e) = db.with_conn(|conn| mark_rappels_annule(conn, rdv_id)) {
        warnings.push(e.message);
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
                prix_centimes: crate::tva::montant_ttc(tarif.prix_centimes, tarif.prix_ttc),
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

fn clear_stripe_link(conn: &Connection, rdv_id: &str) -> Result<Rdv, AppError> {
    let now = now_iso();
    conn.execute(
        "UPDATE rdv SET stripe_url = NULL, stripe_id = NULL, updated_at = ?1 WHERE id = ?2",
        params![now, rdv_id],
    )?;
    fetch_rdv(conn, rdv_id)
}

async fn ensure_stripe_on_rdv(
    db: &DbState,
    settings: &Settings,
    rdv: Rdv,
    recreate: bool,
) -> Result<(Rdv, Vec<String>), AppError> {
    let mut warnings = Vec::new();
    let tarif_data = db.with_conn(|conn| stripe_tarif_data(conn, &rdv, settings))?;
    if tarif_data.is_none() {
        if rdv.stripe_url.is_some() {
            let cleared = db.with_conn(|conn| clear_stripe_link(conn, &rdv.id))?;
            return Ok((cleared, warnings));
        }
        return Ok((rdv, warnings));
    }
    if rdv.stripe_url.is_some() && !recreate {
        return Ok((rdv, warnings));
    }
    let data = tarif_data.unwrap();
    match stripe_create_payment_link(&settings.stripe.secret_key, &data.nom, data.prix_centimes)
        .await
    {
        Ok((stripe_id, stripe_url)) => {
            let updated = db.with_conn(|conn| save_stripe_link(conn, &rdv.id, &stripe_id, &stripe_url))?;
            Ok((updated, warnings))
        }
        Err(e) => {
            eprintln!("stripe: {}", e.message);
            warnings.push(e.message);
            Ok((rdv, warnings))
        }
    }
}

pub fn settings_apply(input: &SettingsSetInput, current: &Settings) -> Settings {
    let token = if input.ntfy_token_clear {
        String::new()
    } else if input.ntfy_token.is_empty() {
        current.ntfy.token.clone()
    } else {
        input.ntfy_token.clone()
    };
    let secret_key = if input.stripe_secret_clear {
        String::new()
    } else if input.stripe_secret_key.is_empty() {
        current.stripe.secret_key.clone()
    } else {
        input.stripe_secret_key.clone()
    };
    Settings {
        ntfy: crate::settings::NtfySettings {
            serveur: input.ntfy_serveur.clone(),
            topic: input.ntfy_topic.clone(),
            token,
            rappel_24h: input.rappel_24h,
            rappel_1h: input.rappel_1h,
        },
        stripe: crate::settings::StripeSettings { secret_key },
        cabinet: crate::settings::CabinetSettings {
            nom: input.cabinet_nom.clone(),
            adresse: input.cabinet_adresse.clone(),
            telephone: input.cabinet_telephone.clone(),
            email: input.cabinet_email.clone(),
            siret: input.cabinet_siret.clone(),
            mention_tva: input.mention_tva.clone(),
            prefixe_numero: input.prefixe_numero.clone(),
        },
    }
}

// --- Tauri commands ---

#[tauri::command(rename_all = "snake_case")]
pub fn settings_get() -> SettingsPublic {
    load_settings().to_public()
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set(input: SettingsSetInput) -> Result<(), String> {
    let current = load_settings();
    let prefixe_numero =
        crate::honoraires::sanitize_prefixe(&input.prefixe_numero).map_err(|e| e.message)?;
    let input = SettingsSetInput {
        prefixe_numero,
        ..input
    };
    let updated = settings_apply(&input, &current);
    save_settings(&updated).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn clients_list(db: tauri::State<'_, DbState>) -> Result<Vec<Client>, String> {
    db.with_conn(repo::clients_list).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn clients_get(id: String, db: tauri::State<'_, DbState>) -> Result<Client, String> {
    db.with_conn(|conn| repo::clients_get(conn, &id)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn clients_delete(id: String, db: tauri::State<'_, DbState>) -> Result<(), String> {
    let settings = load_settings();
    let ntfy_ids =
        db.with_conn(|conn| repo::clients_rappels_ntfy_ids(conn, &id)).map_err(|e| e.message)?;
    db.with_conn(|conn| repo::clients_delete(conn, &id)).map_err(|e| e.message)?;
    for ntfy_id in ntfy_ids {
        if let Err(e) = ntfy_delete(&settings, &ntfy_id).await {
            eprintln!("ntfy delete client: {}", e.message);
        }
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn clients_upsert(
    input: ClientWrite,
    db: tauri::State<'_, DbState>,
) -> Result<Client, String> {
    db.with_conn(|conn| repo::clients_upsert(conn, input))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tarifs_list(db: tauri::State<'_, DbState>) -> Result<Vec<Tarif>, String> {
    db.with_conn(repo::tarifs_list).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tarifs_upsert(
    id: Option<String>,
    nom: String,
    duree_minutes: i64,
    prix_centimes: i64,
    prix_ttc: bool,
    db: tauri::State<'_, DbState>,
) -> Result<Tarif, String> {
    db.with_conn(|conn| {
        repo::tarifs_upsert(
            conn,
            id.as_deref(),
            &nom,
            duree_minutes,
            prix_centimes,
            prix_ttc,
        )
    })
    .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tarifs_set_actif(
    id: String,
    actif: bool,
    db: tauri::State<'_, DbState>,
) -> Result<Tarif, String> {
    db.with_conn(|conn| repo::tarifs_set_actif(conn, &id, actif)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tarifs_delete(id: String, db: tauri::State<'_, DbState>) -> Result<(), String> {
    db.with_conn(|conn| repo::tarifs_delete(conn, &id)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn notes_list(
    client_id: Option<String>,
    perso: bool,
    db: tauri::State<'_, DbState>,
) -> Result<Vec<Note>, String> {
    db.with_conn(|conn| repo::notes_list(conn, client_id, perso)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn notes_upsert(
    id: Option<String>,
    client_id: Option<String>,
    corps: String,
    db: tauri::State<'_, DbState>,
) -> Result<Note, String> {
    db.with_conn(|conn| repo::notes_upsert(conn, id.as_deref(), client_id.as_deref(), &corps))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn notes_delete(id: String, db: tauri::State<'_, DbState>) -> Result<(), String> {
    db.with_conn(|conn| repo::notes_delete(conn, &id)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn rdv_list(
    from: Option<String>,
    to: Option<String>,
    client_id: Option<String>,
    db: tauri::State<'_, DbState>,
) -> Result<Vec<Rdv>, String> {
    db.with_conn(|conn| repo::rdv_list(conn, from.as_deref(), to.as_deref(), client_id.as_deref()))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn rdv_get(id: String, db: tauri::State<'_, DbState>) -> Result<RdvDetail, String> {
    db.with_conn(|conn| repo::rdv_get(conn, &id)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn rdv_create(
    client_id: Option<String>,
    tarif_id: Option<String>,
    debut: String,
    duree_minutes: i64,
    note: Option<String>,
    db: tauri::State<'_, DbState>,
) -> Result<RdvCreateResult, String> {
    let settings = load_settings();
    let now = Utc::now();

    let rdv = db
        .with_conn(|conn| {
            repo::rdv_create(
                conn,
                client_id.as_deref(),
                tarif_id.clone(),
                &debut,
                duree_minutes,
                note.clone(),
            )
        })
        .map_err(|e| e.message)?;

    let mut warnings = Vec::new();
    let (rdv, stripe_warnings) = ensure_stripe_on_rdv(&db, &settings, rdv, false)
        .await
        .map_err(|e| e.message)?;
    warnings.extend(stripe_warnings);
    warnings.extend(ntfy_schedule_rdv(&db, &settings, &rdv.id, now).await);

    Ok(RdvCreateResult { rdv, warnings })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn rdv_update(
    id: String,
    client_id: Option<String>,
    tarif_id: Option<String>,
    debut: String,
    duree_minutes: i64,
    note: Option<String>,
    db: tauri::State<'_, DbState>,
) -> Result<RdvCreateResult, String> {
    let settings = load_settings();
    let now = Utc::now();

    let existing = db.with_conn(|conn| repo::rdv_get(conn, &id)).map_err(|e| e.message)?;
    let debut_changed = match (
        repo::parse_debut_utc(&existing.rdv.debut),
        repo::parse_debut_utc(&debut),
    ) {
        (Ok(a), Ok(b)) => a != b,
        _ => existing.rdv.debut != debut,
    };
    let tarif_changed = existing.rdv.tarif_id != tarif_id;

    let rdv = db
        .with_conn(|conn| {
            repo::rdv_update(
                conn,
                &id,
                client_id.as_deref(),
                tarif_id.clone(),
                &debut,
                duree_minutes,
                note.clone(),
            )
        })
        .map_err(|e| e.message)?;

    let mut warnings = Vec::new();
    let (rdv, stripe_warnings) = ensure_stripe_on_rdv(&db, &settings, rdv, tarif_changed)
        .await
        .map_err(|e| e.message)?;
    warnings.extend(stripe_warnings);
    if debut_changed {
        warnings.extend(cancel_rappels_ntfy(&db, &settings, &id).await);
        warnings.extend(ntfy_schedule_rdv(&db, &settings, &rdv.id, now).await);
    }

    Ok(RdvCreateResult { rdv, warnings })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn rdv_annuler(
    id: String,
    db: tauri::State<'_, DbState>,
) -> Result<RdvCreateResult, String> {
    let settings = load_settings();
    let rdv = db.with_conn(|conn| repo::rdv_annuler(conn, &id)).map_err(|e| e.message)?;
    let warnings = cancel_rappels_ntfy(&db, &settings, &id).await;
    Ok(RdvCreateResult { rdv, warnings })
}

#[tauri::command(rename_all = "snake_case")]
pub fn rdv_set_note(
    id: String,
    note: Option<String>,
    db: tauri::State<'_, DbState>,
) -> Result<Rdv, String> {
    db.with_conn(|conn| repo::rdv_set_note(conn, &id, note)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn rdv_dashboard(db: tauri::State<'_, DbState>) -> Result<Dashboard, String> {
    db.with_conn(|conn| repo::rdv_dashboard(conn, Utc::now())).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn stripe_ensure_link(
    rdv_id: String,
    db: tauri::State<'_, DbState>,
) -> Result<Rdv, String> {
    let settings = load_settings();
    let rdv = db.with_conn(|conn| fetch_rdv(conn, &rdv_id)).map_err(|e| e.message)?;
    let (rdv, warnings) = ensure_stripe_on_rdv(&db, &settings, rdv, true)
        .await
        .map_err(|e| e.message)?;
    if rdv.stripe_url.is_none() {
        if let Some(msg) = warnings.first() {
            return Err(msg.clone());
        }
    }
    Ok(rdv)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ntfy_test() -> Result<(), String> {
    let settings = load_settings();
    crate::ntfy::ntfy_test(&settings)
        .await
        .map_err(|e| e.message)
}

fn pdf_root() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::new("Impossible d'enregistrer le PDF dans le dossier Synapt."))?;
    Ok(home.join("Synapt"))
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_list(
    client_id: Option<String>,
    db: tauri::State<'_, DbState>,
) -> Result<Vec<Honoraire>, String> {
    db.with_conn(|conn| crate::honoraires::honoraires_list(conn, client_id.as_deref()))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_get(
    id: String,
    db: tauri::State<'_, DbState>,
) -> Result<HonoraireDetail, String> {
    db.with_conn(|conn| crate::honoraires::honoraires_get(conn, &id)).map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_create(
    rdv_ids: Vec<String>,
    moyen_paiement: String,
    db: tauri::State<'_, DbState>,
) -> Result<HonoraireDetail, String> {
    let cabinet = load_settings().cabinet;
    let root = pdf_root().map_err(|e| e.message)?;
    db.with_conn(|conn| {
        crate::honoraires::honoraires_create(conn, &cabinet, &rdv_ids, &moyen_paiement, &root)
    })
    .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_ouvrir(
    id: String,
    db: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let root = pdf_root().map_err(|e| e.message)?;
    db.with_conn(|conn| {
        crate::honoraires::honoraires_ouvrir_path(conn, &id, &root)
            .map(|p| p.to_string_lossy().into_owned())
    })
    .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_annuler(
    id: String,
    db: tauri::State<'_, DbState>,
) -> Result<HonoraireDetail, String> {
    let root = pdf_root().map_err(|e| e.message)?;
    db.with_conn(|conn| crate::honoraires::honoraires_annuler(conn, &id, &root))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_rdvs_disponibles(
    client_id: String,
    db: tauri::State<'_, DbState>,
) -> Result<Vec<Rdv>, String> {
    db.with_conn(|conn| crate::honoraires::honoraires_rdvs_disponibles(conn, &client_id))
        .map_err(|e| e.message)
}

#[tauri::command(rename_all = "snake_case")]
pub fn app_close(window: tauri::Window) {
    use tauri::Manager;
    let app = window.app_handle().clone();
    let _ = window.destroy();
    app.exit(0);
}

pub fn run_ntfy_sync(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let db = app.state::<DbState>();
        let settings = load_settings();
        let now = Utc::now();
        let jobs = match db.with_conn(|conn| ntfy_collect_pending(conn, &settings, now, None)) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("ntfy_sync: {}", e.message);
                return;
            }
        };
        for w in ntfy_run_pending_jobs(&db, &settings, jobs).await {
            eprintln!("ntfy_sync: {}", w);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{insert_rappel, mark_rappels_annule, ntfy_sync, settings_apply, SettingsSetInput};
    use crate::db::migrate;
    use crate::error::AppError;
    use crate::models::{Client, ClientWrite, Tarif};
    use crate::ntfy::{NtfyClient, RappelKind};
    use crate::repo::*;
    use crate::settings::Settings;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use rusqlite::{params, Connection};
    use std::sync::Mutex;
    use uuid::Uuid;

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
        clients_upsert(
            conn,
            ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn seed_tarif(conn: &Connection) -> Tarif {
        migrate(conn).unwrap();
        tarifs_upsert(conn, None, "Consultation", 60, 5000, true).unwrap()
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
            cabinet: crate::settings::CabinetSettings::default(),
        }
    }

    #[test]
    fn create_rdv_jitsi_et_planifie() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let tarif = seed_tarif(&conn);
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert!(rdv.jitsi_url.starts_with("https://meet.jit.si/synapt-"));
        assert_eq!(rdv.statut, "planifie");
        assert_eq!(rdv.client_nom, "Alice");
        assert_eq!(rdv.tarif_nom, "Consultation");
    }

    #[test]
    fn rdv_list_et_get_incluent_noms_joints() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let tarif = seed_tarif(&conn);
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let got = rdv_get(&conn, &rdv.id).unwrap();
        assert_eq!(got.rdv.client_nom, "Alice");
        assert_eq!(got.rdv.tarif_nom, "Consultation");
        assert!(got.rappels.is_empty());

        let list = rdv_list(
            &conn,
            Some("2026-09-01T00:00:00Z"),
            Some("2026-09-12T00:00:00Z"),
            None,
        )
        .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].client_nom, "Alice");
        assert_eq!(list[0].tarif_nom, "Consultation");
    }

    #[test]
    fn rdv_list_filtre_par_client_id() {
        let conn = crate::db::open_memory().unwrap();
        let alice = seed_client(&conn);
        let bob = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Bob".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif = seed_tarif(&conn);
        rdv_create(
            &conn,
            Some(&alice.id),
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_create(
            &conn,
            Some(&bob.id),
            Some(tarif.id),
            "2026-09-12T10:00:00Z",
            60,
            None,
        )
        .unwrap();

        let list = rdv_list(&conn, None, None, Some(&alice.id)).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].client_id.as_deref(), Some(alice.id.as_str()));
        assert_eq!(list[0].client_nom, "Alice");
    }

    #[test]
    fn rdv_list_client_inclut_annule() {
        let conn = crate::db::open_memory().unwrap();
        let alice = seed_client(&conn);
        let tarif = seed_tarif(&conn);
        let a = rdv_create(
            &conn,
            Some(&alice.id),
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_create(
            &conn,
            Some(&alice.id),
            Some(tarif.id),
            "2026-09-12T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_annuler(&conn, &a.id).unwrap();

        let list = rdv_list(&conn, None, None, Some(&alice.id)).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|r| r.statut == "annule"));
        assert!(list.iter().any(|r| r.statut == "planifie"));
    }

    #[test]
    fn rappel_reprogram_apres_annule() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            None,
            "2026-09-12T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let echeance1 = Utc.with_ymd_and_hms(2026, 9, 12, 9, 0, 0).unwrap();
        insert_rappel(&conn, &rdv.id, RappelKind::H1, "ntfy-1", echeance1).unwrap();
        mark_rappels_annule(&conn, &rdv.id).unwrap();

        let echeance2 = Utc.with_ymd_and_hms(2026, 9, 12, 8, 0, 0).unwrap();
        insert_rappel(&conn, &rdv.id, RappelKind::H1, "ntfy-2", echeance2).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rappels_ntfy WHERE rdv_id = ?1 AND type = '1h'",
                params![rdv.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let (etat, ntfy_id): (String, String) = conn
            .query_row(
                "SELECT etat, ntfy_id FROM rappels_ntfy WHERE rdv_id = ?1 AND type = '1h'",
                params![rdv.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(etat, "programme");
        assert_eq!(ntfy_id, "ntfy-2");

        rdv_update(
            &conn,
            &rdv.id,
            Some(&client.id),
            None,
            "2026-09-12T14:00:00Z",
            60,
            None,
        )
        .unwrap();

        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rappels_ntfy WHERE rdv_id = ?1 AND type = '1h'",
                params![rdv.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_after, 1);

        let etat_after: String = conn
            .query_row(
                "SELECT etat FROM rappels_ntfy WHERE rdv_id = ?1 AND type = '1h'",
                params![rdv.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(etat_after, "programme");
    }

    #[test]
    fn rdv_sans_tarif_a_tarif_nom_vide() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            None,
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert_eq!(rdv.client_nom, "Alice");
        assert_eq!(rdv.tarif_nom, "");
    }

    #[test]
    fn rdv_sans_client_est_accepte() {
        let conn = crate::db::open_memory().unwrap();
        let tarif = seed_tarif(&conn);
        let rdv = rdv_create(
            &conn,
            None,
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert!(rdv.client_id.is_none());
        assert_eq!(rdv.client_nom, "");
        assert_eq!(rdv.statut, "planifie");

        let got = rdv_get(&conn, &rdv.id).unwrap();
        assert!(got.rdv.client_id.is_none());
        let list = rdv_list(
            &conn,
            Some("2026-09-01T00:00:00Z"),
            Some("2026-09-12T00:00:00Z"),
            None,
        )
        .unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].client_id.is_none());
    }

    #[test]
    fn conflit_horaire_refuse() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        let t = seed_tarif(&conn);
        rdv_create(
            &conn,
            Some(&c.id),
            Some(t.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let err = rdv_create(
            &conn,
            Some(&c.id),
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
            Some(&c.id),
            Some(t.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_annuler(&conn, &a.id).unwrap();
        let b = rdv_create(
            &conn,
            Some(&c.id),
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
        rdv_create(&conn, Some(&client.id), None, &debut, 60, None).unwrap();
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
        rdv_create(&conn, Some(&client.id), None, &debut, 60, None).unwrap();
        let settings = test_settings("topic-test", true);
        let fake = FakeNtfy {
            published: std::sync::Arc::new(Mutex::new(vec![])),
        };
        ntfy_sync(&conn, &settings, now, |_rdv, _kind| fake.clone()).unwrap();
        let published = fake.published.lock().unwrap().clone();
        assert_eq!(published, vec![RappelKind::H1]);
    }

    #[test]
    fn rdv_update_echec_overlap_conserve_rappels() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        let t = seed_tarif(&conn);
        let a = rdv_create(
            &conn,
            Some(&c.id),
            Some(t.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_create(
            &conn,
            Some(&c.id),
            Some(t.id),
            "2026-09-11T11:00:00Z",
            60,
            None,
        )
        .unwrap();
        conn.execute(
            "INSERT INTO rappels_ntfy (id, rdv_id, type, ntfy_id, echeance, etat) VALUES (?1, ?2, '1h', 'fake-id', ?3, 'programme')",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                a.id,
                "2026-09-11T09:00:00Z"
            ],
        )
        .unwrap();

        let err = rdv_update(
            &conn,
            &a.id,
            Some(&c.id),
            None,
            "2026-09-11T11:00:00Z",
            60,
            None,
        )
        .unwrap_err();
        assert!(
            err.message.contains("chevauche") || err.message.contains("horaire"),
            "message: {}",
            err.message
        );

        let etat: String = conn
            .query_row(
                "SELECT etat FROM rappels_ntfy WHERE rdv_id = ?1",
                rusqlite::params![a.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(etat, "programme");
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
            cabinet: crate::settings::CabinetSettings::default(),
        };
        let input = SettingsSetInput {
            ntfy_serveur: "https://ntfy.sh".to_string(),
            ntfy_topic: "new-topic".to_string(),
            ntfy_token: String::new(),
            ntfy_token_clear: false,
            rappel_24h: false,
            rappel_1h: true,
            stripe_secret_key: String::new(),
            stripe_secret_clear: false,
            cabinet_nom: String::new(),
            cabinet_adresse: String::new(),
            cabinet_telephone: String::new(),
            cabinet_email: String::new(),
            cabinet_siret: String::new(),
            mention_tva: crate::settings::mention_tva_defaut().to_string(),
            prefixe_numero: String::new(),
        };
        let updated = settings_apply(&input, &current);
        assert_eq!(updated.ntfy.token, "tok_secret");
        assert_eq!(updated.stripe.secret_key, "sk_test_secret");
        assert_eq!(updated.ntfy.topic, "new-topic");
        assert!(!updated.ntfy.rappel_24h);
    }

    #[test]
    fn clients_upsert_defaults_statut_en_cours() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        assert_eq!(c.statut, "en_cours");
        assert!(c.memo.is_none());
        assert!(c.tarif_id.is_none());
    }

    #[test]
    fn clients_upsert_roundtrip_dossier() {
        let conn = crate::db::open_memory().unwrap();
        migrate(&conn).unwrap();
        let tarif = tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let c = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Alice".into(),
                email: Some("a@b.c".into()),
                telephone: Some("0600000000".into()),
                statut: Some("pause".into()),
                memo: Some("  WhatsApp  ".into()),
                tarif_id: Some(tarif.id.clone()),
                date_naissance: Some("1990-05-12".into()),
                urgence_nom: Some("Paul".into()),
                urgence_telephone: Some("0700000000".into()),
                orientation: Some("medecin".into()),
                frequence: Some("hebdo".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(c.statut, "pause");
        assert_eq!(c.memo.as_deref(), Some("WhatsApp"));
        assert_eq!(c.tarif_id.as_deref(), Some(tarif.id.as_str()));
        assert_eq!(c.date_naissance.as_deref(), Some("1990-05-12"));
        assert_eq!(c.urgence_nom.as_deref(), Some("Paul"));
        assert_eq!(c.urgence_telephone.as_deref(), Some("0700000000"));
        assert_eq!(c.orientation.as_deref(), Some("medecin"));
        assert_eq!(c.frequence.as_deref(), Some("hebdo"));
        let got = clients_get(&conn, &c.id).unwrap();
        assert_eq!(got.statut, "pause");
        assert_eq!(got.tarif_id, c.tarif_id);
    }

    #[test]
    fn clients_upsert_rejects_invalid_statut() {
        let conn = crate::db::open_memory().unwrap();
        migrate(&conn).unwrap();
        let err = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Alice".into(),
                statut: Some("archive".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.message.contains("Statut"));
    }

    #[test]
    fn clients_upsert_rejects_unknown_tarif() {
        let conn = crate::db::open_memory().unwrap();
        migrate(&conn).unwrap();
        let err = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Alice".into(),
                tarif_id: Some("missing".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.message.contains("tarif introuvable"));
    }

    #[test]
    fn clients_delete_fiche_seule() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        clients_delete(&conn, &c.id).unwrap();
        assert!(clients_get(&conn, &c.id).is_err());
    }

    #[test]
    fn clients_delete_cascade_notes_rdv_annule_rappels() {
        let conn = crate::db::open_memory().unwrap();
        let alice = seed_client(&conn);
        let bob = clients_upsert(
            &conn,
            ClientWrite {
                nom: "Bob".into(),
                ..Default::default()
            },
        )
        .unwrap();
        notes_upsert(&conn, None, Some(&alice.id), "note alice").unwrap();
        notes_upsert(&conn, None, Some(&bob.id), "note bob").unwrap();
        let rdv = rdv_create(
            &conn,
            Some(&alice.id),
            None,
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        rdv_annuler(&conn, &rdv.id).unwrap();
        conn.execute(
            "INSERT INTO rappels_ntfy (id, rdv_id, type, ntfy_id, echeance, etat) VALUES (?1, ?2, '1h', 'fake-id', ?3, 'annule')",
            rusqlite::params![Uuid::new_v4().to_string(), rdv.id, "2026-09-11T09:00:00Z"],
        )
        .unwrap();

        clients_delete(&conn, &alice.id).unwrap();

        assert!(clients_get(&conn, &alice.id).is_err());
        assert_eq!(clients_get(&conn, &bob.id).unwrap().nom, "Bob");
        assert!(notes_list(&conn, Some(alice.id.clone()), false)
            .unwrap()
            .is_empty());
        assert_eq!(
            notes_list(&conn, Some(bob.id.clone()), false)
                .unwrap()
                .len(),
            1
        );
        assert!(rdv_list(&conn, None, None, Some(&alice.id))
            .unwrap()
            .is_empty());
        let rappels: i64 = conn
            .query_row("SELECT COUNT(*) FROM rappels_ntfy", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rappels, 0);
    }

    #[test]
    fn clients_delete_refuse_rdv_planifie() {
        let conn = crate::db::open_memory().unwrap();
        let c = seed_client(&conn);
        rdv_create(&conn, Some(&c.id), None, "2026-09-11T10:00:00Z", 60, None).unwrap();
        let err = clients_delete(&conn, &c.id).unwrap_err();
        assert!(err.message.contains("rendez-vous prévus"));
        assert_eq!(clients_get(&conn, &c.id).unwrap().nom, "Alice");
    }

    #[test]
    fn clients_delete_inconnu() {
        let conn = crate::db::open_memory().unwrap();
        migrate(&conn).unwrap();
        let err = clients_delete(&conn, "missing").unwrap_err();
        assert_eq!(err.message, "client introuvable");
    }

    #[test]
    fn settings_clear_efface_secrets() {
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
            cabinet: crate::settings::CabinetSettings::default(),
        };
        let input = SettingsSetInput {
            ntfy_serveur: current.ntfy.serveur.clone(),
            ntfy_topic: current.ntfy.topic.clone(),
            ntfy_token: String::new(),
            ntfy_token_clear: true,
            rappel_24h: true,
            rappel_1h: true,
            stripe_secret_key: String::new(),
            stripe_secret_clear: true,
            cabinet_nom: String::new(),
            cabinet_adresse: String::new(),
            cabinet_telephone: String::new(),
            cabinet_email: String::new(),
            cabinet_siret: String::new(),
            mention_tva: crate::settings::mention_tva_defaut().to_string(),
            prefixe_numero: String::new(),
        };
        let updated = settings_apply(&input, &current);
        assert!(updated.ntfy.token.is_empty());
        assert!(updated.stripe.secret_key.is_empty());
    }

    #[test]
    fn rdv_set_note_maj_uniquement_note() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            None,
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        let updated = rdv_set_note(&conn, &rdv.id, Some("note test".into())).unwrap();
        assert_eq!(updated.note.as_deref(), Some("note test"));
        rdv_annuler(&conn, &rdv.id).unwrap();
        let err = rdv_set_note(&conn, &rdv.id, Some("x".into())).unwrap_err();
        assert!(err.message.contains("annulé"));
    }

    #[test]
    fn rdv_create_refuse_debut_invalide() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let err = rdv_create(&conn, Some(&client.id), None, "pas-une-date", 60, None).unwrap_err();
        assert!(err.message.contains("valide"));
    }

    #[test]
    fn rdv_update_tarif_change_efface_stripe() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        let t1 = tarifs_upsert(&conn, None, "A", 60, 5000, true).unwrap();
        let t2 = tarifs_upsert(&conn, None, "B", 60, 6000, true).unwrap();
        let rdv = rdv_create(
            &conn,
            Some(&client.id),
            Some(t1.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        conn.execute(
            "UPDATE rdv SET stripe_url = 'https://stripe.test', stripe_id = 'pl_test' WHERE id = ?1",
            params![rdv.id],
        )
        .unwrap();
        let updated = rdv_update(
            &conn,
            &rdv.id,
            Some(&client.id),
            Some(t2.id.clone()),
            "2026-09-11T10:00:00Z",
            60,
            None,
        )
        .unwrap();
        assert_eq!(updated.tarif_id.as_deref(), Some(t2.id.as_str()));
        assert!(updated.stripe_url.is_none());
        assert!(updated.stripe_id.is_none());
    }

    #[test]
    fn rdv_dashboard_formats_debut_mixtes() {
        let conn = crate::db::open_memory().unwrap();
        let client = seed_client(&conn);
        rdv_create(
            &conn,
            Some(&client.id),
            None,
            "2026-09-11T10:00:00.000Z",
            60,
            None,
        )
        .unwrap();
        let now = Utc.with_ymd_and_hms(2026, 9, 11, 8, 0, 0).unwrap();
        let dash = rdv_dashboard(&conn, now).unwrap();
        assert_eq!(dash.aujourdhui.len(), 1);
    }
}
