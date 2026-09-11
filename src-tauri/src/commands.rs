use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::db::migrate;
use crate::error::AppError;
use crate::models::{Client, Dashboard, Note, Rdv, Tarif};
use crate::overlap::overlaps;

pub fn jitsi_url(id: &str) -> String {
    format!("https://meet.jit.si/synapt-{id}")
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn seed_client(conn: &Connection) -> Client {
        migrate(conn).unwrap();
        clients_upsert(conn, None, "Alice", None, None).unwrap()
    }

    fn seed_tarif(conn: &Connection) -> Tarif {
        migrate(conn).unwrap();
        tarifs_upsert(conn, None, "Consultation", 60, 5000).unwrap()
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
}
