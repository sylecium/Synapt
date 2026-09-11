use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::AppError;

const MIGRATION: &str = r"
CREATE TABLE IF NOT EXISTS clients (
  id TEXT PRIMARY KEY,
  nom TEXT NOT NULL,
  email TEXT,
  telephone TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tarifs (
  id TEXT PRIMARY KEY,
  nom TEXT NOT NULL,
  duree_minutes INTEGER NOT NULL CHECK (duree_minutes > 0),
  prix_centimes INTEGER NOT NULL CHECK (prix_centimes >= 0),
  actif INTEGER NOT NULL DEFAULT 1,
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

pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(MIGRATION)?;
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
}
