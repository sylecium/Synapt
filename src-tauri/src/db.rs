use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::AppError;

const MIGRATION: &str = r"
CREATE TABLE IF NOT EXISTS tarifs (
  id TEXT PRIMARY KEY,
  nom TEXT NOT NULL,
  duree_minutes INTEGER NOT NULL CHECK (duree_minutes > 0),
  prix_centimes INTEGER NOT NULL CHECK (prix_centimes >= 0),
  actif INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS clients (
  id TEXT PRIMARY KEY,
  nom TEXT NOT NULL,
  email TEXT,
  telephone TEXT,
  statut TEXT NOT NULL DEFAULT 'en_cours',
  memo TEXT,
  tarif_id TEXT REFERENCES tarifs(id),
  date_naissance TEXT,
  urgence_nom TEXT,
  urgence_telephone TEXT,
  orientation TEXT,
  frequence TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS rdv (
  id TEXT PRIMARY KEY,
  client_id TEXT NOT NULL REFERENCES clients(id),
  tarif_id TEXT REFERENCES tarifs(id),
  debut TEXT NOT NULL,
  duree_minutes INTEGER NOT NULL CHECK (duree_minutes > 0),
  jitsi_url TEXT NOT NULL,
  stripe_url TEXT,
  stripe_id TEXT,
  note TEXT,
  statut TEXT NOT NULL CHECK (statut IN ('planifie', 'annule')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  client_id TEXT REFERENCES clients(id),
  corps TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS rappels_ntfy (
  id TEXT PRIMARY KEY,
  rdv_id TEXT NOT NULL REFERENCES rdv(id),
  type TEXT NOT NULL CHECK (type IN ('24h', '1h')),
  ntfy_id TEXT,
  echeance TEXT NOT NULL,
  etat TEXT NOT NULL CHECK (etat IN ('programme', 'annule')),
  UNIQUE (rdv_id, type)
);
";

pub fn open_memory() -> Result<Connection, AppError> {
    Connection::open_in_memory().map_err(AppError::from)
}

pub fn open_file(path: &Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Connection::open(path).map_err(AppError::from)
}

pub fn db_path() -> Result<PathBuf, AppError> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| AppError::new("impossible de determiner le repertoire de donnees"))?;
    Ok(data_dir.join("synapt").join("synapt.db"))
}

const CLIENT_ALTERS: &[(&str, &str)] = &[
    ("statut", "TEXT NOT NULL DEFAULT 'en_cours'"),
    ("memo", "TEXT"),
    ("tarif_id", "TEXT REFERENCES tarifs(id)"),
    ("date_naissance", "TEXT"),
    ("urgence_nom", "TEXT"),
    ("urgence_telephone", "TEXT"),
    ("orientation", "TEXT"),
    ("frequence", "TEXT"),
];

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(cols)
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    ddl: &str,
) -> Result<(), AppError> {
    let cols = table_columns(conn, table)?;
    if cols.iter().any(|c| c == column) {
        return Ok(());
    }
    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {ddl}"),
        [],
    )?;
    Ok(())
}

pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    conn.execute_batch(MIGRATION)?;
    for (column, ddl) in CLIENT_ALTERS {
        add_column_if_missing(conn, "clients", column, ddl)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_rdv_table() {
        let conn = open_memory().unwrap();
        migrate(&conn).unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(names.contains(&"rdv".to_string()));
    }

    #[test]
    fn migrate_adds_client_dossier_columns_on_old_schema() {
        let conn = open_memory().unwrap();
        conn.execute_batch(
            r"
            CREATE TABLE clients (
              id TEXT PRIMARY KEY,
              nom TEXT NOT NULL,
              email TEXT,
              telephone TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            INSERT INTO clients (id, nom, created_at, updated_at)
            VALUES ('c1', 'Alice', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
            ",
        )
        .unwrap();

        migrate(&conn).unwrap();
        migrate(&conn).unwrap();

        let cols = table_columns(&conn, "clients").unwrap();
        for name in [
            "statut",
            "memo",
            "tarif_id",
            "date_naissance",
            "urgence_nom",
            "urgence_telephone",
            "orientation",
            "frequence",
        ] {
            assert!(cols.contains(&name.to_string()), "missing {name}");
        }

        let statut: String = conn
            .query_row("SELECT statut FROM clients WHERE id = 'c1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(statut, "en_cours");
    }
}
