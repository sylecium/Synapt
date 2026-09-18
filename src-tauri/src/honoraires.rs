use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Honoraire, HonoraireDetail, HonoraireLigne, Rdv};
use crate::repo::{self, now_iso};
use crate::settings::CabinetSettings;

const HONORAIRE_SELECT: &str = "\
SELECT id, numero, client_id, client_nom, client_date_naissance, client_adresse, \
cabinet_nom, cabinet_adresse, cabinet_telephone, cabinet_email, cabinet_siret, mention_tva, \
moyen_paiement, statut, total_centimes, annee, seq, pdf_relatif, created_at \
FROM honoraires";

const MOYENS: &[&str] = &["especes", "cheque", "cb", "stripe"];

pub fn pdf_abs(pdf_root: &Path, relatif: &str) -> PathBuf {
    pdf_root.join(relatif)
}

fn none_if_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn row_to_honoraire(row: &Row<'_>) -> Result<Honoraire, rusqlite::Error> {
    Ok(Honoraire {
        id: row.get(0)?,
        numero: row.get(1)?,
        client_id: row.get(2)?,
        client_nom: row.get(3)?,
        client_date_naissance: row.get(4)?,
        client_adresse: row.get(5)?,
        cabinet_nom: row.get(6)?,
        cabinet_adresse: row.get(7)?,
        cabinet_telephone: row.get(8)?,
        cabinet_email: row.get(9)?,
        cabinet_siret: row.get(10)?,
        mention_tva: row.get(11)?,
        moyen_paiement: row.get(12)?,
        statut: row.get(13)?,
        total_centimes: row.get(14)?,
        annee: row.get(15)?,
        seq: row.get(16)?,
        pdf_relatif: row.get(17)?,
        created_at: row.get(18)?,
    })
}

fn row_to_ligne(row: &Row<'_>) -> Result<HonoraireLigne, rusqlite::Error> {
    Ok(HonoraireLigne {
        id: row.get(0)?,
        honoraire_id: row.get(1)?,
        rdv_id: row.get(2)?,
        debut: row.get(3)?,
        duree_minutes: row.get(4)?,
        tarif_nom: row.get(5)?,
        prix_centimes: row.get(6)?,
        actif: row.get::<_, i64>(7)? != 0,
    })
}

pub fn honoraires_get(conn: &Connection, id: &str) -> Result<HonoraireDetail, AppError> {
    repo::ensure_migrated(conn)?;
    let sql = format!("{HONORAIRE_SELECT} WHERE id = ?1");
    let honoraire = conn
        .query_row(&sql, params![id], row_to_honoraire)
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                AppError::new("Note d'honoraires introuvable.")
            } else {
                AppError::from(e)
            }
        })?;
    let mut stmt = conn.prepare(
        "SELECT id, honoraire_id, rdv_id, debut, duree_minutes, tarif_nom, prix_centimes, actif \
         FROM honoraire_lignes WHERE honoraire_id = ?1 ORDER BY debut",
    )?;
    let lignes = stmt
        .query_map(params![id], row_to_ligne)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HonoraireDetail { honoraire, lignes })
}

pub fn honoraires_get_for_rdv(
    conn: &Connection,
    rdv_id: &str,
) -> Result<Option<Honoraire>, AppError> {
    repo::ensure_migrated(conn)?;
    let sql = "\
        SELECT h.id, h.numero, h.client_id, h.client_nom, h.client_date_naissance, h.client_adresse, \
        h.cabinet_nom, h.cabinet_adresse, h.cabinet_telephone, h.cabinet_email, h.cabinet_siret, h.mention_tva, \
        h.moyen_paiement, h.statut, h.total_centimes, h.annee, h.seq, h.pdf_relatif, h.created_at \
        FROM honoraires h \
        JOIN honoraire_lignes l ON l.honoraire_id = h.id \
        WHERE l.rdv_id = ?1 AND l.actif = 1 AND h.statut = 'emise' \
        LIMIT 1";
    let honoraire = conn
        .query_row(sql, params![rdv_id], row_to_honoraire)
        .optional()
        .map_err(AppError::from)?;
    Ok(honoraire)
}

fn fetch_rdv_honoraires(conn: &Connection, id: &str) -> Result<Rdv, AppError> {
    match repo::fetch_rdv(conn, id) {
        Ok(r) => Ok(r),
        Err(e) if e.message.contains("Query returned no rows") => {
            Err(AppError::new("Rendez-vous introuvable."))
        }
        Err(e) => Err(e),
    }
}

fn tarif_prix_ttc(conn: &Connection, tarif_id: &Option<String>) -> Result<i64, AppError> {
    match tarif_id {
        None => Ok(0),
        Some(id) => {
            let tarif = repo::tarifs_get(conn, id)?;
            Ok(crate::tva::montant_ttc(tarif.prix_centimes, tarif.prix_ttc))
        }
    }
}

fn rdv_deja_facture(conn: &Connection, rdv_id: &str) -> Result<bool, AppError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM honoraire_lignes WHERE rdv_id = ?1 AND actif = 1",
        params![rdv_id],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

pub fn honoraires_create(
    conn: &Connection,
    cabinet: &CabinetSettings,
    rdv_ids: &[String],
    moyen: &str,
    pdf_root: &Path,
) -> Result<HonoraireDetail, AppError> {
    repo::ensure_migrated(conn)?;

    if rdv_ids.is_empty() {
        return Err(AppError::new("Choisissez au moins une séance."));
    }
    let mut seen = HashSet::new();
    for id in rdv_ids {
        if !seen.insert(id.as_str()) {
            return Err(AppError::new("Cette séance a déjà une note d'honoraires."));
        }
    }
    if cabinet.nom.trim().is_empty() {
        return Err(AppError::new(
            "Indiquez le nom du cabinet dans les réglages.",
        ));
    }
    if !MOYENS.contains(&moyen) {
        return Err(AppError::new("Moyen de paiement inconnu."));
    }

    let prefixe = sanitize_prefixe(&cabinet.prefixe_numero)?;
    let annee = Local::now().year();

    conn.execute_batch("BEGIN IMMEDIATE")?;
    let created_id = (|| -> Result<String, AppError> {
        let mut rdvs = Vec::with_capacity(rdv_ids.len());
        for id in rdv_ids {
            let rdv = fetch_rdv_honoraires(conn, id)?;
            if rdv.statut != "planifie" {
                return Err(AppError::new("Impossible d'inclure un rendez-vous annulé."));
            }
            if rdv_deja_facture(conn, id)? {
                return Err(AppError::new("Cette séance a déjà une note d'honoraires."));
            }
            rdvs.push(rdv);
        }

        let client_id = rdvs[0].client_id.clone().ok_or_else(|| {
            AppError::new("Attribuez un client au rendez-vous pour émettre une note d'honoraires.")
        })?;
        if rdvs
            .iter()
            .any(|r| r.client_id.as_deref() != Some(client_id.as_str()))
        {
            return Err(AppError::new(
                "Une note ne peut concerner qu'un seul client.",
            ));
        }

        rdvs.sort_by(|a, b| a.debut.cmp(&b.debut));

        let seq: i32 = conn.query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM honoraires WHERE annee = ?1",
            params![annee],
            |row| row.get(0),
        )?;

        let client = repo::clients_get(conn, &client_id)?;
        let numero = format_numero(&prefixe, annee, seq);
        let pdf_relatif = format!("honoraires/{}.pdf", numero);
        let honoraire_id = Uuid::new_v4().to_string();
        let created_at = now_iso();

        let mut lignes: Vec<(String, &Rdv, String, i64)> = Vec::new();
        let mut total_centimes: i64 = 0;
        for rdv in &rdvs {
            let tarif_nom = if rdv.tarif_nom.is_empty() {
                "Séance".to_string()
            } else {
                rdv.tarif_nom.clone()
            };
            let prix = tarif_prix_ttc(conn, &rdv.tarif_id)?;
            total_centimes += prix;
            lignes.push((Uuid::new_v4().to_string(), rdv, tarif_nom, prix));
        }

        let cabinet_nom = cabinet.nom.trim().to_string();
        conn.execute(
            "INSERT INTO honoraires (
                id, numero, client_id, client_nom, client_date_naissance, client_adresse,
                cabinet_nom, cabinet_adresse, cabinet_telephone, cabinet_email, cabinet_siret,
                mention_tva, moyen_paiement, statut, total_centimes, annee, seq, pdf_relatif, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'emise', ?14, ?15, ?16, ?17, ?18)",
            params![
                honoraire_id,
                numero,
                client.id,
                client.nom,
                client.date_naissance,
                client.adresse,
                cabinet_nom,
                none_if_empty(&cabinet.adresse),
                none_if_empty(&cabinet.telephone),
                none_if_empty(&cabinet.email),
                none_if_empty(&cabinet.siret),
                cabinet.mention_tva.trim(),
                moyen,
                total_centimes,
                annee,
                seq,
                pdf_relatif,
                created_at,
            ],
        )?;

        for (ligne_id, rdv, tarif_nom, prix) in &lignes {
            conn.execute(
                "INSERT INTO honoraire_lignes (
                    id, honoraire_id, rdv_id, debut, duree_minutes, tarif_nom, prix_centimes, actif
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                params![
                    ligne_id,
                    honoraire_id,
                    rdv.id,
                    rdv.debut,
                    rdv.duree_minutes,
                    tarif_nom,
                    prix,
                ],
            )?;
        }

        let detail = honoraires_get(conn, &honoraire_id)?;
        let pdf_path = pdf_abs(pdf_root, &pdf_relatif);
        if crate::honoraires_pdf::write_pdf(&detail, &pdf_path).is_err() {
            return Err(AppError::new(
                "Impossible d'enregistrer le PDF dans le dossier Synapt.",
            ));
        }

        Ok(honoraire_id)
    })();

    match created_id {
        Ok(id) => {
            conn.execute_batch("COMMIT")?;
            honoraires_get(conn, &id)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

pub fn honoraires_list(
    conn: &Connection,
    client_id: Option<&str>,
) -> Result<Vec<Honoraire>, AppError> {
    repo::ensure_migrated(conn)?;
    let sql = match client_id {
        Some(_) => format!("{HONORAIRE_SELECT} WHERE client_id = ?1 ORDER BY created_at DESC"),
        None => format!("{HONORAIRE_SELECT} ORDER BY created_at DESC"),
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = match client_id {
        Some(cid) => stmt.query_map(params![cid], row_to_honoraire)?,
        None => stmt.query_map([], row_to_honoraire)?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn honoraires_ouvrir_path(
    conn: &Connection,
    id: &str,
    pdf_root: &Path,
) -> Result<PathBuf, AppError> {
    let detail = honoraires_get(conn, id)?;
    let path = pdf_abs(pdf_root, &detail.honoraire.pdf_relatif);
    if !path.is_file() && crate::honoraires_pdf::write_pdf(&detail, &path).is_err() {
        return Err(AppError::new("Le PDF n'a pas pu être ouvert."));
    }
    Ok(path)
}

pub fn honoraires_annuler(
    conn: &Connection,
    id: &str,
    pdf_root: &Path,
) -> Result<HonoraireDetail, AppError> {
    repo::ensure_migrated(conn)?;
    let current = honoraires_get(conn, id)?;
    if current.honoraire.statut == "annulee" {
        return Err(AppError::new("Cette note est déjà annulée."));
    }

    conn.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| -> Result<(), AppError> {
        conn.execute(
            "UPDATE honoraires SET statut = 'annulee' WHERE id = ?1",
            params![id],
        )?;
        conn.execute(
            "UPDATE honoraire_lignes SET actif = 0 WHERE honoraire_id = ?1",
            params![id],
        )?;
        let detail = honoraires_get(conn, id)?;
        let pdf_path = pdf_abs(pdf_root, &detail.honoraire.pdf_relatif);
        if crate::honoraires_pdf::write_pdf(&detail, &pdf_path).is_err() {
            return Err(AppError::new(
                "Impossible d'enregistrer le PDF dans le dossier Synapt.",
            ));
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            honoraires_get(conn, id)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

pub fn honoraires_rdvs_disponibles(
    conn: &Connection,
    client_id: &str,
) -> Result<Vec<Rdv>, AppError> {
    repo::ensure_migrated(conn)?;
    let sql = format!(
        "{} WHERE rdv.statut = 'planifie' AND rdv.client_id = ?1 \
         AND rdv.id NOT IN (SELECT rdv_id FROM honoraire_lignes WHERE actif = 1) \
         ORDER BY rdv.debut",
        repo::RDV_SELECT
    );
    let mut stmt = conn.prepare(&sql)?;
    let rdvs = stmt
        .query_map(params![client_id], repo::row_to_rdv)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rdvs)
}

pub fn sanitize_prefixe(raw: &str) -> Result<String, AppError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.len() > 12
        || !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::new(
            "Le préfixe du numéro n'accepte que lettres, chiffres, _ et - (12 caractères max).",
        ));
    }
    Ok(s.to_string())
}

pub fn format_numero(prefixe: &str, annee: i32, seq: i32) -> String {
    let corps = format!("{annee}-{seq:04}");
    if prefixe.is_empty() {
        corps
    } else if prefixe.ends_with('-') {
        format!("{prefixe}{corps}")
    } else {
        format!("{prefixe}-{corps}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numero_sans_prefixe() {
        assert_eq!(format_numero("", 2026, 1), "2026-0001");
    }

    #[test]
    fn numero_prefixe_nh_sans_tiret() {
        assert_eq!(format_numero("NH", 2026, 12), "NH-2026-0012");
    }

    #[test]
    fn numero_prefixe_deja_tiret() {
        assert_eq!(format_numero("NH-", 2026, 1), "NH-2026-0001");
    }

    #[test]
    fn sanitize_vide() {
        assert_eq!(sanitize_prefixe("").unwrap(), "");
        assert_eq!(sanitize_prefixe("  ").unwrap(), "");
    }

    #[test]
    fn sanitize_ok() {
        assert_eq!(sanitize_prefixe("NH_1").unwrap(), "NH_1");
    }

    #[test]
    fn sanitize_refuse_slash() {
        let err = sanitize_prefixe("a/b").unwrap_err();
        assert!(err.message.contains("préfixe"));
    }

    use chrono::Datelike;
    use rusqlite::Connection;

    fn cabinet_ok() -> crate::settings::CabinetSettings {
        crate::settings::CabinetSettings {
            nom: "Cabinet Test".into(),
            ..Default::default()
        }
    }

    fn seed_rdv(
        conn: &Connection,
        client_id: &str,
        tarif_id: Option<String>,
        debut: &str,
    ) -> crate::models::Rdv {
        crate::repo::rdv_create(conn, Some(client_id), tarif_id, debut, 60, None).unwrap()
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("synapt-hon-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(p.join("honoraires")).unwrap();
        p
    }

    #[test]
    fn create_une_seance() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                date_naissance: Some("1990-05-12".into()),
                adresse: Some("1 rue A".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let rdv = seed_rdv(&conn, &client.id, Some(tarif.id), "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let detail =
            honoraires_create(&conn, &cabinet_ok(), std::slice::from_ref(&rdv.id), "especes", &dir).unwrap();
        let annee = chrono::Local::now().year();
        assert_eq!(detail.honoraire.numero, format!("{annee}-0001"));
        assert_eq!(detail.honoraire.seq, 1);
        assert_eq!(detail.honoraire.statut, "emise");
        assert_eq!(detail.honoraire.client_nom, "Alice");
        assert_eq!(detail.honoraire.client_adresse.as_deref(), Some("1 rue A"));
        assert_eq!(detail.honoraire.total_centimes, 5000);
        assert_eq!(detail.lignes.len(), 1);
        assert_eq!(detail.lignes[0].prix_centimes, 5000);
        assert!(detail.lignes[0].actif);
        let path = dir.join(&detail.honoraire.pdf_relatif);
        assert!(path.is_file());
    }

    #[test]
    fn create_refuse_rdv_sans_client() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let rdv =
            crate::repo::rdv_create(&conn, None, None, "2026-09-11T10:00:00Z", 60, None).unwrap();
        let dir = tempfile_dir();
        let err = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap_err();
        assert!(err.message.contains("client"));
    }

    #[test]
    fn deuxieme_note_incremente() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let a = seed_rdv(
            &conn,
            &client.id,
            Some(tarif.id.clone()),
            "2026-09-11T10:00:00Z",
        );
        let b = seed_rdv(&conn, &client.id, Some(tarif.id), "2026-09-12T10:00:00Z");
        let dir = tempfile_dir();
        let d1 = honoraires_create(&conn, &cabinet_ok(), &[a.id], "cb", &dir).unwrap();
        let d2 = honoraires_create(&conn, &cabinet_ok(), &[b.id], "cb", &dir).unwrap();
        assert_eq!(d2.honoraire.seq, d1.honoraire.seq + 1);
    }

    #[test]
    fn prefixe_nh() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let mut cab = cabinet_ok();
        cab.prefixe_numero = "NH".into();
        let d = honoraires_create(&conn, &cab, &[rdv.id], "cheque", &dir).unwrap();
        let annee = chrono::Local::now().year();
        assert_eq!(d.honoraire.numero, format!("NH-{annee}-0001"));
    }

    #[test]
    fn refuse_cabinet_sans_nom() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let err = honoraires_create(
            &conn,
            &Default::default(),
            &[rdv.id],
            "especes",
            &tempfile_dir(),
        )
        .unwrap_err();
        assert_eq!(err.message, "Indiquez le nom du cabinet dans les réglages.");
    }

    #[test]
    fn refuse_rdv_deja_emis() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        honoraires_create(&conn, &cabinet_ok(), std::slice::from_ref(&rdv.id), "especes", &dir).unwrap();
        let err = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap_err();
        assert_eq!(err.message, "Cette séance a déjà une note d'honoraires.");
    }

    #[test]
    fn refuse_clients_melanges() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let a = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "A".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let b = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "B".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let ra = seed_rdv(&conn, &a.id, None, "2026-09-11T10:00:00Z");
        let rb = seed_rdv(&conn, &b.id, None, "2026-09-12T10:00:00Z");
        let err = honoraires_create(
            &conn,
            &cabinet_ok(),
            &[ra.id, rb.id],
            "especes",
            &tempfile_dir(),
        )
        .unwrap_err();
        assert_eq!(err.message, "Une note ne peut concerner qu'un seul client.");
    }

    #[test]
    fn refuse_vide() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let err =
            honoraires_create(&conn, &cabinet_ok(), &[], "especes", &tempfile_dir()).unwrap_err();
        assert_eq!(err.message, "Choisissez au moins une séance.");
    }

    #[test]
    fn regroupement_deux_rdv() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let t1 = crate::repo::tarifs_upsert(&conn, None, "A", 60, 4000, true).unwrap();
        let t2 = crate::repo::tarifs_upsert(&conn, None, "B", 45, 3000, true).unwrap();
        let r2 = seed_rdv(&conn, &client.id, Some(t2.id), "2026-09-12T10:00:00Z");
        let r1 = seed_rdv(&conn, &client.id, Some(t1.id), "2026-09-11T10:00:00Z");
        let d = honoraires_create(
            &conn,
            &cabinet_ok(),
            &[r2.id, r1.id],
            "stripe",
            &tempfile_dir(),
        )
        .unwrap();
        assert_eq!(d.lignes.len(), 2);
        assert_eq!(d.honoraire.total_centimes, 7000);
        assert!(d.lignes[0].debut < d.lignes[1].debut);
    }

    #[test]
    fn rdv_sans_tarif() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let d =
            honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &tempfile_dir()).unwrap();
        assert_eq!(d.lignes[0].tarif_nom, "Séance");
        assert_eq!(d.lignes[0].prix_centimes, 0);
        assert_eq!(d.honoraire.total_centimes, 0);
    }

    #[test]
    fn tarif_ht_facture_en_ttc() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, false).unwrap();
        let rdv = seed_rdv(&conn, &client.id, Some(tarif.id), "2026-09-11T10:00:00Z");
        let d =
            honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &tempfile_dir()).unwrap();
        assert_eq!(d.lignes[0].prix_centimes, 6000);
        assert_eq!(d.honoraire.total_centimes, 6000);
    }

    #[test]
    fn annuler_libere_rdv_sans_reutiliser_numero() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let a = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let b = seed_rdv(&conn, &client.id, None, "2026-09-12T10:00:00Z");
        let dir = tempfile_dir();
        let d1 = honoraires_create(&conn, &cabinet_ok(), std::slice::from_ref(&a.id), "especes", &dir).unwrap();
        honoraires_annuler(&conn, &d1.honoraire.id, &dir).unwrap();
        let dispo = honoraires_rdvs_disponibles(&conn, &client.id).unwrap();
        assert!(dispo.iter().any(|r| r.id == a.id));
        let d2 = honoraires_create(&conn, &cabinet_ok(), &[a.id], "especes", &dir).unwrap();
        assert_eq!(d2.honoraire.seq, d1.honoraire.seq + 1);
        let _ = b;
    }

    #[test]
    fn pdf_header_and_accents() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Léa".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
        let bytes = std::fs::read(dir.join(&d.honoraire.pdf_relatif)).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 200);
        assert_ne!(bytes.as_slice(), b"%PDF-stub\n");
    }

    fn pdftotext(path: &std::path::Path) -> String {
        let out = std::process::Command::new("pdftotext")
            .args(["-layout", path.to_str().unwrap(), "-"])
            .output()
            .expect("pdftotext");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[test]
    fn pdf_facture_tva20_et_mentions() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Léa Martin".into(),
                adresse: Some("12 rue des Lilas, 75011 Paris".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let rdv = seed_rdv(&conn, &client.id, Some(tarif.id), "2026-09-11T10:00:00Z");
        let mut cab = cabinet_ok();
        cab.adresse = "1 avenue de la République, 33000 Bordeaux".into();
        cab.siret = "40483304800022".into();
        cab.telephone = "05 56 00 00 00".into();
        cab.email = "cabinet@example.com".into();
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cab, &[rdv.id], "especes", &dir).unwrap();
        let text = pdftotext(&dir.join(&d.honoraire.pdf_relatif));
        assert!(
            text.contains("Léa Martin"),
            "client absent du PDF: {text:?}"
        );
        assert!(
            text.contains("Cabinet Test"),
            "prestataire absent: {text:?}"
        );
        assert!(text.contains("FACTURE"), "titre FACTURE absent: {text:?}");
        assert!(text.contains("TVA 20"), "TVA 20 % absente: {text:?}");
        assert!(text.contains("41,67"), "HT 41,67 absent: {text:?}");
        assert!(text.contains("8,33"), "TVA 8,33 absente: {text:?}");
        assert!(text.contains("50,00"), "TTC 50,00 absent: {text:?}");
        assert!(text.contains("Acquittée"), "acquittée absente: {text:?}");
        assert!(text.contains("SIRET"), "SIRET absent: {text:?}");
        assert!(
            text.contains("N° TVA") || text.contains("TVA intra"),
            "TVA intra absente: {text:?}"
        );
        assert!(
            text.contains("encaissements"),
            "mention CGI encaissements absente: {text:?}"
        );
        assert!(
            text.contains("cinquante euros"),
            "montant en lettres absent: {text:?}"
        );
    }

    #[test]
    fn ouvrir_regenere_si_manquant() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
        let path = dir.join(&d.honoraire.pdf_relatif);
        std::fs::remove_file(&path).unwrap();
        let opened = honoraires_ouvrir_path(&conn, &d.honoraire.id, &dir).unwrap();
        assert_eq!(opened, path);
        assert!(path.is_file());
    }

    #[test]
    fn ouvrir_ne_regenere_pas_si_existant() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
        let path = dir.join(&d.honoraire.pdf_relatif);
        assert!(path.is_file());
        std::fs::write(&path, b"SENTINEL_NON_ECRASE").unwrap();
        let opened = honoraires_ouvrir_path(&conn, &d.honoraire.id, &dir).unwrap();
        assert_eq!(opened, path);
        let content = std::fs::read(&path).unwrap();
        assert_eq!(content, b"SENTINEL_NON_ECRASE");
    }

    #[test]
    fn annuler_deux_fois() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
        honoraires_annuler(&conn, &d.honoraire.id, &dir).unwrap();
        let err = honoraires_annuler(&conn, &d.honoraire.id, &dir).unwrap_err();
        assert_eq!(err.message, "Cette note est déjà annulée.");
    }

    #[test]
    fn pdf_adresse_multiligne_preserve_les_lignes() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Léa Martin".into(),
                adresse: Some("12 rue des Lilas\nBâtiment B\n75011 Paris".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
        let mut cab = cabinet_ok();
        cab.adresse = "1 avenue de la République\n33000 Bordeaux".into();
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cab, &[rdv.id], "especes", &dir).unwrap();
        let text = pdftotext(&dir.join(&d.honoraire.pdf_relatif));
        assert!(text.contains("12 rue des Lilas"), "manque ligne 1 client: {text:?}");
        assert!(text.contains("Bâtiment B"), "manque ligne 2 client: {text:?}");
        assert!(text.contains("75011 Paris"), "manque ligne 3 client: {text:?}");
        assert!(text.contains("1 avenue de la République"), "manque ligne 1 cabinet: {text:?}");
        assert!(text.contains("33000 Bordeaux"), "manque ligne 2 cabinet: {text:?}");
    }

    #[test]
    fn pdf_saut_page_totaux_multi_lignes() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Léa Martin".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let mut rdv_ids = Vec::new();
        for i in 0..33 {
            let day = (i / 4) + 1;
            let hour = 8 + (i % 4) * 2;
            let iso = format!("2026-10-{day:02}T{hour:02}:00:00Z");
            let rdv = seed_rdv(&conn, &client.id, Some(tarif.id.clone()), &iso);
            rdv_ids.push(rdv.id);
        }
        let dir = tempfile_dir();
        let d = honoraires_create(&conn, &cabinet_ok(), &rdv_ids, "cb", &dir).unwrap();
        let text = pdftotext(&dir.join(&d.honoraire.pdf_relatif));
        let pages: Vec<&str> = text.split('\x0c').collect();
        let non_empty_pages: Vec<&str> = pages.into_iter().filter(|p| !p.trim().is_empty()).collect();
        assert_eq!(non_empty_pages.len(), 2, "doit comporter exactement 2 pages: {text:?}");
        let page2 = non_empty_pages[1];
        assert!(page2.contains("Total HT"), "page 2 doit contenir les totaux: {page2:?}");
        assert!(page2.contains("Total TTC"), "page 2 doit contenir Total TTC: {page2:?}");
        assert!(page2.contains("Arrêtée la présente facture"), "page 2 doit contenir la mention légale: {page2:?}");
        assert!(page2.contains("Pour acquit"), "page 2 doit contenir Pour acquit: {page2:?}");
    }

    #[test]
    fn get_for_rdv_nominal_et_annule() {
        let conn = crate::db::open_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        let client = crate::repo::clients_upsert(
            &conn,
            crate::models::ClientWrite {
                nom: "Alice".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let tarif =
            crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000, true).unwrap();
        let rdv1 = seed_rdv(&conn, &client.id, Some(tarif.id.clone()), "2026-10-15T09:00:00Z");
        let rdv2 = seed_rdv(&conn, &client.id, Some(tarif.id.clone()), "2026-10-16T09:00:00Z");

        assert!(honoraires_get_for_rdv(&conn, &rdv1.id).unwrap().is_none());

        let dir = tempfile_dir();
        let detail = honoraires_create(
            &conn,
            &cabinet_ok(),
            std::slice::from_ref(&rdv1.id),
            "especes",
            &dir,
        )
        .unwrap();

        let found = honoraires_get_for_rdv(&conn, &rdv1.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, detail.honoraire.id);

        assert!(honoraires_get_for_rdv(&conn, &rdv2.id).unwrap().is_none());

        honoraires_annuler(&conn, &detail.honoraire.id, &dir).unwrap();
        assert!(honoraires_get_for_rdv(&conn, &rdv1.id).unwrap().is_none());
    }
}
