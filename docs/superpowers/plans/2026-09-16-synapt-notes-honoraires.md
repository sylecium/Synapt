# Notes d’honoraires PDF Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Sous-agents d’implémentation : **Composer 2.5 normal**, pas fast. Ne pas modifier ce plan pendant l’implémentation ; cocher les `- [ ]` uniquement. Ne pas recréer ces todos. Commits seulement si l’humain l’a demandé dans la session (sinon skip la step commit).

**Goal:** Émettre, ouvrir et annuler des notes d’honoraires PDF B2C (1 ou N séances), figées, numérotées, stockées dans `~/Synapt/honoraires/`.

**Architecture:** Logique et PDF en Rust (`invoke`). SQLite = vérité. Fichiers PDF dérivés. Frontend Svelte : `/honoraires`, fiche client, panneau RDV, réglages cabinet. Pas de SQL ni d’IO disque dans le webview hors `openPath`.

**Tech Stack:** Tauri 2, Svelte 5, rusqlite, `printpdf`, police DejaVu embarquée, plugin opener, Bun.

**Spec:** `docs/superpowers/specs/2026-09-16-synapt-notes-honoraires-design.md`

## Global Constraints

- Cible Debian 13, solo local, pas de cloud.
- Prix en centimes `i64`, IDs UUID v4 texte, datetimes UTC ISO 8601, affichage local.
- Frontend : ni SQL, ni Stripe, ni ntfy, ni lecture directe de `~/Synapt/`.
- UI français, compréhensible, pas de jargon IPC/SQL, pas de tiret cadratin (utiliser `-`).
- Commandes Tauri `rename_all = "snake_case"`.
- Pas de Factur-X, email, avoir, logo, ADELI, webhook Stripe.
- `commands.rs` déjà trop gros : logique honoraires dans `honoraires.rs` + `honoraires_pdf.rs`. Wrappers fins seulement dans `commands.rs`.
- Tests frontend dans `tests/` (pas dans `src/`).
- Année du numéro = fuseau **local** (`chrono::Local`).

## File map

```
src-tauri/Cargo.toml
src-tauri/fonts/DejaVuSans.ttf
src-tauri/fonts/DejaVuSans-Bold.ttf
src-tauri/src/lib.rs
src-tauri/src/db.rs
src-tauri/src/models.rs
src-tauri/src/settings.rs
src-tauri/src/repo.rs
src-tauri/src/commands.rs
src-tauri/src/honoraires.rs          (create)
src-tauri/src/honoraires_pdf.rs      (create)
src-tauri/capabilities/default.json
src/lib/types.ts
src/lib/api.ts
src/lib/errors.ts
src/lib/format.ts
src/lib/components/AppSidebar.svelte
src/lib/components/ClientForm.svelte
src/lib/components/HonoraireDialog.svelte   (create)
src/lib/components/HonoraireContextMenu.svelte (create)
src/lib/components/RdvContextMenu.svelte
src/lib/components/RdvPanel.svelte
src/routes/honoraires/+page.svelte          (create)
src/routes/clients/[id]/+page.svelte
src/routes/reglages/+page.svelte
tests/honorairesFormat.test.ts              (create)
tests/errors.test.ts
```

---

### Task 1: Numéro + réglages cabinet

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/commands.rs` (`SettingsSetInput`, `settings_apply`, `test_settings`)
- Create: `src-tauri/src/honoraires.rs` (helpers purs seulement)
- Modify: `src-tauri/src/lib.rs` (`pub mod honoraires;`)

**Interfaces:**
- Consumes: `Settings` existant
- Produces:
  - `CabinetSettings { nom, adresse, telephone, email, siret, mention_tva, prefixe_numero: String }`
  - `Settings.cabinet: CabinetSettings` avec `#[serde(default)]`
  - `SettingsPublic` + champs cabinet (tous en clair)
  - `fn format_numero(prefixe: &str, annee: i32, seq: i32) -> String`
  - `fn sanitize_prefixe(raw: &str) -> Result<String, AppError>`
  - `settings::mention_tva_defaut() -> &'static str` = `"TVA non applicable, art. 261-4-1° du CGI"`
  - `SettingsSetInput` + champs cabinet
  - `settings_apply` copie le cabinet après `sanitize_prefixe`

- [ ] **Step 1: Test `format_numero` / `sanitize_prefixe` (échec : module absent)**

Dans `src-tauri/src/honoraires.rs` sous `#[cfg(test)]` (fichier créé avec les tests d’abord, fns commentées ou absentes) :

```rust
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
}
```

- [ ] **Step 2: Lancer le test, constater l’échec**

Run: `cd src-tauri && cargo test --lib honoraires::tests -- --nocapture`
Expected: FAIL (module / fonctions absents)

- [ ] **Step 3: Implémenter les helpers + CabinetSettings**

`honoraires.rs` :

```rust
use crate::error::AppError;

pub fn sanitize_prefixe(raw: &str) -> Result<String, AppError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.len() > 12 || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
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
```

`settings.rs` : ne pas importer `honoraires` (cycle). Ajouter :

```rust
pub fn mention_tva_defaut() -> &'static str {
    "TVA non applicable, art. 261-4-1° du CGI"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CabinetSettings {
    #[serde(default)]
    pub nom: String,
    #[serde(default)]
    pub adresse: String,
    #[serde(default)]
    pub telephone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub siret: String,
    #[serde(default = "default_mention_tva")]
    pub mention_tva: String,
    #[serde(default)]
    pub prefixe_numero: String,
}

fn default_mention_tva() -> String {
    mention_tva_defaut().to_string()
}

impl Default for CabinetSettings {
    fn default() -> Self {
        Self {
            nom: String::new(),
            adresse: String::new(),
            telephone: String::new(),
            email: String::new(),
            siret: String::new(),
            mention_tva: default_mention_tva(),
            prefixe_numero: String::new(),
        }
    }
}
```

Sur `Settings` : `#[serde(default)] pub cabinet: CabinetSettings`.
`Default` de `Settings` : `cabinet: CabinetSettings::default()`.
`SettingsPublic` : ajouter `cabinet_nom, cabinet_adresse, cabinet_telephone, cabinet_email, cabinet_siret, mention_tva, prefixe_numero`.
`to_public` : les remplir depuis `self.cabinet`.

`SettingsSetInput` : mêmes 7 champs cabinet.
`settings_apply` : `prefixe` via `sanitize_prefixe` (si Err, `settings_set` mappe déjà `e.message`). Pour rester pur, `settings_apply` devient `Result<Settings, AppError>` **ou** sanitize dans `settings_set` seulement.

Décision : `settings_apply` reste pur ; `settings_set` appelle `sanitize_prefixe(&input.prefixe_numero)?` puis `settings_apply`.

Mettre à jour `test_settings` dans `commands.rs` : `cabinet: crate::settings::CabinetSettings::default()`.

- [ ] **Step 4: Relancer les tests**

Run: `cd src-tauri && cargo test --lib honoraires::tests settings::tests`
Expected: PASS. Puis `cargo test --lib` pour le reste (fixer `Settings { ... }` incomplets).

- [ ] **Step 5: Commit** (skip si l’humain n’a pas demandé)

```bash
git add src-tauri/src/honoraires.rs src-tauri/src/settings.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: réglages cabinet et numérotation des notes d'honoraires"
```

---

### Task 2: Migration SQLite

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/models.rs`

**Interfaces:**
- Consumes: `migrate()` existant
- Produces: tables `honoraires`, `honoraire_lignes` ; colonne `clients.adresse` ; types `Honoraire`, `HonoraireLigne`, `HonoraireDetail`

- [ ] **Step 1: Test migrate**

Ajouter dans `db.rs` `#[cfg(test)]` :

```rust
#[test]
fn migrate_creates_honoraires_and_client_adresse() {
    let conn = open_memory().unwrap();
    migrate(&conn).unwrap();
    migrate(&conn).unwrap();
    let cols = table_columns(&conn, "clients").unwrap();
    assert!(cols.contains(&"adresse".to_string()));
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .unwrap();
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(names.contains(&"honoraires".to_string()));
    assert!(names.contains(&"honoraire_lignes".to_string()));
}

#[test]
fn migrate_adds_adresse_on_old_clients() {
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
        ",
    )
    .unwrap();
    migrate(&conn).unwrap();
    let cols = table_columns(&conn, "clients").unwrap();
    assert!(cols.contains(&"adresse".to_string()));
}
```

- [ ] **Step 2: Run, expect FAIL** (tables absentes)

Run: `cd src-tauri && cargo test --lib db::tests::migrate_creates_honoraires -- --nocapture`

- [ ] **Step 3: DDL + models**

Dans `MIGRATION` (fin du batch) :

```sql
CREATE TABLE IF NOT EXISTS honoraires (
  id TEXT PRIMARY KEY,
  numero TEXT NOT NULL UNIQUE,
  client_id TEXT NOT NULL REFERENCES clients(id),
  client_nom TEXT NOT NULL,
  client_date_naissance TEXT,
  client_adresse TEXT,
  cabinet_nom TEXT NOT NULL,
  cabinet_adresse TEXT,
  cabinet_telephone TEXT,
  cabinet_email TEXT,
  cabinet_siret TEXT,
  mention_tva TEXT NOT NULL,
  moyen_paiement TEXT NOT NULL CHECK (moyen_paiement IN ('especes', 'cheque', 'cb', 'stripe')),
  statut TEXT NOT NULL CHECK (statut IN ('emise', 'annulee')),
  total_centimes INTEGER NOT NULL CHECK (total_centimes >= 0),
  annee INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  pdf_relatif TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (annee, seq)
);
CREATE TABLE IF NOT EXISTS honoraire_lignes (
  id TEXT PRIMARY KEY,
  honoraire_id TEXT NOT NULL REFERENCES honoraires(id),
  rdv_id TEXT NOT NULL REFERENCES rdv(id),
  debut TEXT NOT NULL,
  duree_minutes INTEGER NOT NULL,
  tarif_nom TEXT NOT NULL,
  prix_centimes INTEGER NOT NULL,
  actif INTEGER NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_honoraire_lignes_rdv_actif
  ON honoraire_lignes(rdv_id) WHERE actif = 1;
```

`CLIENT_ALTERS` : ajouter `("adresse", "TEXT")`.

`models.rs` :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Honoraire {
    pub id: String,
    pub numero: String,
    pub client_id: String,
    pub client_nom: String,
    pub client_date_naissance: Option<String>,
    pub client_adresse: Option<String>,
    pub cabinet_nom: String,
    pub cabinet_adresse: Option<String>,
    pub cabinet_telephone: Option<String>,
    pub cabinet_email: Option<String>,
    pub cabinet_siret: Option<String>,
    pub mention_tva: String,
    pub moyen_paiement: String,
    pub statut: String,
    pub total_centimes: i64,
    pub annee: i32,
    pub seq: i32,
    pub pdf_relatif: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonoraireLigne {
    pub id: String,
    pub honoraire_id: String,
    pub rdv_id: String,
    pub debut: String,
    pub duree_minutes: i64,
    pub tarif_nom: String,
    pub prix_centimes: i64,
    pub actif: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonoraireDetail {
    pub honoraire: Honoraire,
    pub lignes: Vec<HonoraireLigne>,
}
```

`Client` / `ClientWrite` : `pub adresse: Option<String>` (Default sur Write).

- [ ] **Step 4: cargo test db::tests**

Expected: PASS. `cargo test --lib` : corriger `row_to_client` / `CLIENT_SELECT` / `clients_upsert` SQL pour inclure `adresse` (sinon compile OK mais SELECT cassé). Faire le SELECT/INSERT/UPDATE `adresse` **dans cette tâche** pour que `clients_get` ne casse pas :

- `CLIENT_SELECT` ajoute `adresse`
- `row_to_client` mappe `adresse`
- `clients_upsert` INSERT/UPDATE colonne `adresse` via `trim_opt(write.adresse)`

Test existant `clients_upsert_roundtrip_dossier` : ajouter `adresse: Some("1 rue A".into())` et `assert_eq!(c.adresse.as_deref(), Some("1 rue A"));`

- [ ] **Step 5: Commit** (skip si non demandé)

---

### Task 3: `honoraires_create` (1 séance + snapshots + fichier)

**Files:**
- Modify: `src-tauri/src/honoraires.rs`
- Create: `src-tauri/src/honoraires_pdf.rs` (stub qui écrit des bytes `%PDF-stub\n` pour cette tâche ; PDF réel = Task 7)

**Interfaces:**
- Consumes: `CabinetSettings`, `format_numero`, tables Task 2, `repo::fetch_rdv`, `repo::now_iso`
- Produces:
  - `pub fn honoraires_create(conn, cabinet: &CabinetSettings, rdv_ids: &[String], moyen: &str, pdf_root: &Path) -> Result<HonoraireDetail, AppError>`
  - `pub fn honoraires_get(conn, id) -> Result<HonoraireDetail, AppError>`
  - `pub fn pdf_abs(pdf_root: &Path, relatif: &str) -> PathBuf`
  - Fichier `{pdf_root}/{pdf_relatif}` créé

- [ ] **Step 1: Test create 1 séance**

```rust
fn cabinet_ok() -> crate::settings::CabinetSettings {
    crate::settings::CabinetSettings {
        nom: "Cabinet Test".into(),
        ..Default::default()
    }
}

fn seed_rdv(conn: &Connection, client_id: &str, tarif_id: Option<String>, debut: &str) -> crate::models::Rdv {
    crate::repo::rdv_create(conn, client_id, tarif_id, debut, 60, None).unwrap()
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
    let tarif = crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000).unwrap();
    let rdv = seed_rdv(&conn, &client.id, Some(tarif.id), "2026-09-11T10:00:00Z");
    let dir = tempfile_dir();
    let detail = honoraires_create(&conn, &cabinet_ok(), &[rdv.id.clone()], "especes", &dir).unwrap();
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

fn tempfile_dir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("synapt-hon-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(p.join("honoraires")).unwrap();
    p
}
```

Ajouter `use chrono::Datelike;` dans le module de test.

- [ ] **Step 2: cargo test honoraires::tests::create_une_seance** → FAIL

- [ ] **Step 3: Implémenter create**

Règles (transaction `BEGIN IMMEDIATE`) :

1. `rdv_ids` vide → `"Choisissez au moins une séance."`
2. doublons → même message ou `"Cette séance a déjà une note d'honoraires."`
3. `cabinet.nom.trim().is_empty()` → `"Indiquez le nom du cabinet dans les réglages."`
4. `moyen` ∈ especes|cheque|cb|stripe sinon `"Moyen de paiement inconnu."`
5. Charger chaque RDV (`fetch_rdv`). Introuvable → `"Rendez-vous introuvable."`
6. `statut != planifie` → `"Impossible d'inclure un rendez-vous annulé."`
7. Tous le même `client_id` sinon `"Une note ne peut concerner qu'un seul client."`
8. `SELECT COUNT(*) FROM honoraire_lignes WHERE rdv_id = ? AND actif = 1` > 0 → `"Cette séance a déjà une note d'honoraires."`
9. `annee = Local::now().year() as i32` ; `seq = COALESCE(MAX(seq),0)+1 WHERE annee = ?`
10. Snapshot client via `clients_get` ; cabinet depuis l’argument (trim, empty → None pour optionnels)
11. Ligne : `tarif_nom` = `rdv.tarif_nom` si non vide sinon `"Séance"` ; `prix_centimes` = prix tarif ou 0 (JOIN déjà dans fetch ; si `tarif_id` none, 0)
12. Trier lignes par `debut`
13. Insert honoraires + lignes `actif = 1`
14. `write_pdf_stub` dans `honoraires_pdf.rs` : `create_dir_all` parent, write `b"%PDF-stub\n"`
15. Commit. Si PDF/IO fail : rollback + `"Impossible d'enregistrer le PDF dans le dossier Synapt."`

`pdf_relatif` = `format!("honoraires/{}.pdf", numero)` (`numero` = `format_numero(&cabinet.prefixe_numero, annee, seq)`). `prefixe` déjà sanitizé en réglages ; ici `sanitize_prefixe` à nouveau (idempotent).

Pour le prix : `SELECT prix_centimes FROM tarifs WHERE id = rdv.tarif_id` si Some.

- [ ] **Step 4: test PASS**

Run: `cd src-tauri && cargo test --lib honoraires::tests::create_une_seance`

- [ ] **Step 5: Commit** (skip si non demandé)

---

### Task 4: Refus create + 2e numéro + préfixe

**Files:**
- Modify: `src-tauri/src/honoraires.rs` (tests + éventuels gardes manquants)

**Interfaces:**
- Consumes: `honoraires_create`
- Produces: mêmes erreurs que la spec

- [ ] **Step 1: Tests**

```rust
#[test]
fn deuxieme_note_incremente() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let tarif = crate::repo::tarifs_upsert(&conn, None, "Consultation", 60, 5000).unwrap();
    let a = seed_rdv(&conn, &client.id, Some(tarif.id.clone()), "2026-09-11T10:00:00Z");
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
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
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
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let err = honoraires_create(&conn, &Default::default(), &[rdv.id], "especes", &tempfile_dir()).unwrap_err();
    assert_eq!(err.message, "Indiquez le nom du cabinet dans les réglages.");
}

#[test]
fn refuse_rdv_deja_emis() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let dir = tempfile_dir();
    honoraires_create(&conn, &cabinet_ok(), &[rdv.id.clone()], "especes", &dir).unwrap();
    let err = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap_err();
    assert_eq!(err.message, "Cette séance a déjà une note d'honoraires.");
}

#[test]
fn refuse_clients_melanges() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let a = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "A".into(), ..Default::default() }).unwrap();
    let b = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "B".into(), ..Default::default() }).unwrap();
    let ra = seed_rdv(&conn, &a.id, None, "2026-09-11T10:00:00Z");
    let rb = seed_rdv(&conn, &b.id, None, "2026-09-12T10:00:00Z");
    let err = honoraires_create(&conn, &cabinet_ok(), &[ra.id, rb.id], "especes", &tempfile_dir()).unwrap_err();
    assert_eq!(err.message, "Une note ne peut concerner qu'un seul client.");
}

#[test]
fn refuse_vide() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let err = honoraires_create(&conn, &cabinet_ok(), &[], "especes", &tempfile_dir()).unwrap_err();
    assert_eq!(err.message, "Choisissez au moins une séance.");
}
```

- [ ] **Step 2: Run tests → FAIL sur les cas non gérés, puis implémenter les gardes manquants**

- [ ] **Step 3: PASS** `cargo test --lib honoraires::tests`

- [ ] **Step 4: Commit** (skip si non demandé)

---

### Task 5: Regroupement + RDV sans tarif

**Files:**
- Modify: `src-tauri/src/honoraires.rs`

**Interfaces:**
- Consumes: `honoraires_create`
- Produces: total = somme ; 2 lignes triées par `debut` ; sans tarif → `Séance` / 0

- [ ] **Step 1: Tests**

```rust
#[test]
fn regroupement_deux_rdv() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let t1 = crate::repo::tarifs_upsert(&conn, None, "A", 60, 4000).unwrap();
    let t2 = crate::repo::tarifs_upsert(&conn, None, "B", 45, 3000).unwrap();
    let r2 = seed_rdv(&conn, &client.id, Some(t2.id), "2026-09-12T10:00:00Z");
    let r1 = seed_rdv(&conn, &client.id, Some(t1.id), "2026-09-11T10:00:00Z");
    let d = honoraires_create(&conn, &cabinet_ok(), &[r2.id, r1.id], "stripe", &tempfile_dir()).unwrap();
    assert_eq!(d.lignes.len(), 2);
    assert_eq!(d.honoraire.total_centimes, 7000);
    assert!(d.lignes[0].debut < d.lignes[1].debut);
}

#[test]
fn rdv_sans_tarif() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &tempfile_dir()).unwrap();
    assert_eq!(d.lignes[0].tarif_nom, "Séance");
    assert_eq!(d.lignes[0].prix_centimes, 0);
    assert_eq!(d.honoraire.total_centimes, 0);
}
```

- [ ] **Step 2-4:** FAIL / implémenter tri + fallback / PASS / commit skippable

---

### Task 6: Annuler + numéro non réutilisé + rdv disponibles

**Files:**
- Modify: `src-tauri/src/honoraires.rs`
- Modify: `src-tauri/src/honoraires_pdf.rs` (`write_pdf` relit le snapshot ; si `statut == "annulee"` le stub contient `ANNULEE`)

**Interfaces:**
- Produces:
  - `honoraires_annuler(conn, id, pdf_root) -> Result<HonoraireDetail, AppError>`
  - `honoraires_list(conn, client_id: Option<&str>) -> Result<Vec<Honoraire>, AppError>` (ORDER BY `created_at` DESC)
  - `honoraires_rdvs_disponibles(conn, client_id) -> Result<Vec<Rdv>, AppError>`

- [ ] **Step 1: Tests**

```rust
#[test]
fn annuler_libere_rdv_sans_reutiliser_numero() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let a = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let b = seed_rdv(&conn, &client.id, None, "2026-09-12T10:00:00Z");
    let dir = tempfile_dir();
    let d1 = honoraires_create(&conn, &cabinet_ok(), &[a.id.clone()], "especes", &dir).unwrap();
    honoraires_annuler(&conn, &d1.honoraire.id, &dir).unwrap();
    let dispo = honoraires_rdvs_disponibles(&conn, &client.id).unwrap();
    assert!(dispo.iter().any(|r| r.id == a.id));
    let d2 = honoraires_create(&conn, &cabinet_ok(), &[a.id], "especes", &dir).unwrap();
    assert_eq!(d2.honoraire.seq, d1.honoraire.seq + 1);
    let _ = b;
}

#[test]
fn annuler_deux_fois() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let dir = tempfile_dir();
    let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
    honoraires_annuler(&conn, &d.honoraire.id, &dir).unwrap();
    let err = honoraires_annuler(&conn, &d.honoraire.id, &dir).unwrap_err();
    assert_eq!(err.message, "Cette note est déjà annulée.");
}
```

- [ ] **Step 2: FAIL puis implémenter**

`honoraires_annuler` : get ; si `annulee` → erreur ; UPDATE statut ; `UPDATE honoraire_lignes SET actif = 0 WHERE honoraire_id = ?` ; `write_pdf` ; return get.

`rdvs_disponibles` : `rdv.statut = planifie` AND `rdv.client_id = ?` AND id NOT IN (lignes actif=1). Réutiliser `RDV_SELECT`.

- [ ] **Step 3: PASS** `cargo test --lib honoraires::tests`

- [ ] **Step 4: Commit** skippable

---

### Task 7: PDF réel + ouvrir si fichier manquant

**Files:**
- Modify: `src-tauri/Cargo.toml` (`printpdf = "0.9"`)
- Create: `src-tauri/fonts/DejaVuSans.ttf` et `DejaVuSans-Bold.ttf` (copier depuis `/usr/share/fonts/truetype/dejavu/` ; si absent : `sudo apt-get install -y fonts-dejavu-core`)
- Modify: `src-tauri/src/honoraires_pdf.rs`
- Modify: `src-tauri/src/honoraires.rs` (`honoraires_ouvrir_path`)

**Interfaces:**
- Produces: `pub fn write_pdf(detail: &HonoraireDetail, dest: &Path) -> Result<(), AppError>`
- `pub fn honoraires_ouvrir_path(conn, id, pdf_root) -> Result<PathBuf, AppError>`
- Police : `include_bytes!("../fonts/DejaVuSans.ttf")`

API printpdf (0.9) :

```rust
use printpdf::*;
let mut doc = PdfDocument::new("Note d'honoraires");
let font = ParsedFont::from_bytes(include_bytes!("../fonts/DejaVuSans.ttf"), 0, &mut vec![]).unwrap();
let font_id = doc.add_font(&font);
// pages A4 Mm(210.0) x Mm(297.0)
// Op::StartTextSection / SetFont / ShowText / EndTextSection
let bytes = doc.with_pages(pages).save(&PdfSaveOptions { subset_fonts: true, ..Default::default() }, &mut vec![]);
```

Contenu (champs vides omis, pas de « - ») :

1. `Note d'honoraires` + numero + date locale de `created_at`
2. Cabinet / client
3. Lignes : date locale, horaire, tarif, durée, montant `XX,XX €`
4. Total
5. `Acquittée` + libellé moyen
6. mention TVA
7. `Signature`
8. Si `statut == "annulee"` : texte `ANNULÉE` (grande taille, centre)

Libellés moyen (Rust) : Espèces / Chèque / Carte / Stripe.

- [ ] **Step 1: Tests**

```rust
#[test]
fn pdf_header_and_accents() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Léa".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let dir = tempfile_dir();
    let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
    let bytes = std::fs::read(dir.join(&d.honoraire.pdf_relatif)).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn ouvrir_regenere_si_manquant() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    let dir = tempfile_dir();
    let d = honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &dir).unwrap();
    let path = dir.join(&d.honoraire.pdf_relatif);
    std::fs::remove_file(&path).unwrap();
    let opened = honoraires_ouvrir_path(&conn, &d.honoraire.id, &dir).unwrap();
    assert_eq!(opened, path);
    assert!(path.is_file());
}
```

- [ ] **Step 2: FAIL (stub `%PDF-stub` n’est pas un vrai PDF ou `ouvrir` absent)**

- [ ] **Step 3: Remplacer le stub par `write_pdf` réel. `honoraires_ouvrir_path` : get ; si `!path.is_file()` → `write_pdf` ; Ok(path). IO fail → `"Le PDF n'a pas pu être ouvert."`**

Si `ParsedFont::from_bytes` échoue, message `"Impossible d'enregistrer le PDF dans le dossier Synapt."`

- [ ] **Step 4: `cargo test --lib honoraires::tests` PASS**

- [ ] **Step 5: Commit** skippable

---

### Task 8: `clients_delete` refuse si note

**Files:**
- Modify: `src-tauri/src/repo.rs` (`clients_delete`)
- Modify: `src-tauri/src/commands.rs` tests existants si besoin

**Interfaces:**
- Consumes: table `honoraires`
- Produces: erreur `"Ce client a encore des notes d'honoraires."` avant le check RDV planifiés (ou après, peu importe tant que c’est avant le DELETE)

- [ ] **Step 1: Test dans `commands.rs` tests ou `honoraires.rs`**

```rust
#[test]
fn clients_delete_refuse_si_honoraire() {
    let conn = crate::db::open_memory().unwrap();
    crate::db::migrate(&conn).unwrap();
    let client = crate::repo::clients_upsert(&conn, crate::models::ClientWrite { nom: "Alice".into(), ..Default::default() }).unwrap();
    let rdv = seed_rdv(&conn, &client.id, None, "2026-09-11T10:00:00Z");
    honoraires_create(&conn, &cabinet_ok(), &[rdv.id], "especes", &tempfile_dir()).unwrap();
    crate::repo::rdv_annuler(&conn, &rdv.id).ok(); // si la fn exige planifie, skip
    let err = crate::repo::clients_delete(&conn, &client.id).unwrap_err();
    assert_eq!(err.message, "Ce client a encore des notes d'honoraires.");
}
```

Note : `clients_delete` refuse déjà les RDV `planifie`. Pour ce test, **annuler le RDV d’abord** (`repo::rdv_annuler`) **sans** annuler la note (la note `emise` garde `rdv_id` et `actif=1` : le RDV est encore `planifie` jusqu’à `rdv_annuler`). Ordre du test :

1. create honoraire (RDV reste `planifie`)
2. `rdv_annuler` → delete client échouerait encore sur honoraires **si** on check honoraires. Mais `rdv_annuler` sur un RDV lié : FK `honoraire_lignes.rdv_id REFERENCES rdv(id)` **bloque l’annulation?** Non, annuler ne DELETE pas le RDV.
3. `clients_delete` : encore `planifie`? Après `rdv_annuler` statut `annule`. Check honoraires doit déclencher.

Si `rdv_annuler` n’existe pas comme `repo::rdv_annuler`, utiliser le chemin déjà testé dans `commands.rs` (`rdv_annuler`).

- [ ] **Step 2-4:** FAIL / ajouter COUNT honoraires / PASS / commit skippable

Ne pas DELETE les honoraires en cascade.

---

### Task 9: Commands Tauri + capabilities

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (`generate_handler!`)
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Produces (tous `rename_all = "snake_case"`) :
  - `honoraires_list(client_id: Option<String>)`
  - `honoraires_get(id: String)`
  - `honoraires_create(rdv_ids: Vec<String>, moyen_paiement: String)`
  - `honoraires_ouvrir(id: String) -> String` (chemin absolu)
  - `honoraires_annuler(id: String)`
  - `honoraires_rdvs_disponibles(client_id: String)`
- `clients_upsert` + param `adresse: Option<String>`
- `pdf_root()` = `dirs::home_dir()?.join("Synapt")` ; erreur home absente → `"Impossible d'enregistrer le PDF dans le dossier Synapt."`
- `settings_set` sanitize préfixe

`default.json` ajouter :

```json
{
  "identifier": "opener:allow-open-path",
  "allow": [
    { "path": "$HOME/Synapt/honoraires" },
    { "path": "$HOME/Synapt/honoraires/**" }
  ]
}
```

Wrappers :

```rust
fn pdf_root() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::new("Impossible d'enregistrer le PDF dans le dossier Synapt."))?;
    Ok(home.join("Synapt"))
}

#[tauri::command(rename_all = "snake_case")]
pub fn honoraires_create(rdv_ids: Vec<String>, moyen_paiement: String) -> Result<HonoraireDetail, String> {
    let cabinet = load_settings().cabinet;
    let root = pdf_root().map_err(|e| e.message)?;
    with_db(|conn| crate::honoraires::honoraires_create(conn, &cabinet, &rdv_ids, &moyen_paiement, &root)).map_err(|e| e.message)
}
```

Même motif pour list/get/annuler/ouvrir/rdvs_disponibles.

- [ ] **Step 1: `cd src-tauri && cargo check`**
Expected: SUCCESS

- [ ] **Step 2: `cd src-tauri && cargo test --lib`**
Expected: PASS

- [ ] **Step 3: Commit** skippable

---

### Task 10: Types / api / erreurs / format frontend

**Files:**
- Modify: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/errors.ts`, `src/lib/format.ts`
- Create: `tests/honorairesFormat.test.ts`
- Modify: `tests/errors.test.ts`

**Interfaces:**
- Produces: types `MoyenPaiement`, `Honoraire`, `HonoraireLigne`, `HonoraireDetail` alignés snake_case
- `Client.adresse: string | null`
- `SettingsPublic` cabinet fields
- `SettingsSetInput` cabinet fields
- `honorairesList`, `honorairesGet`, `honorairesCreate`, `honorairesOuvrir`, `honorairesAnnuler`, `honorairesRdvsDisponibles`
- `clientsUpsert` + `adresse`
- `MOYEN_PAIEMENT_LABELS` + `moyenPaiementLabel`
- `userMessage` laisse passer les phrases spec (< 120 chars). Ajouter filets si le rust contient `préfixe` / `honoraires` déjà en français.

- [ ] **Step 1: Test format**

```ts
import { describe, expect, test } from 'bun:test';
import { moyenPaiementLabel } from '../src/lib/format';

describe('moyenPaiementLabel', () => {
	test('libellés', () => {
		expect(moyenPaiementLabel('especes')).toBe('Espèces');
		expect(moyenPaiementLabel('cheque')).toBe('Chèque');
		expect(moyenPaiementLabel('cb')).toBe('Carte');
		expect(moyenPaiementLabel('stripe')).toBe('Stripe');
	});
});
```

- [ ] **Step 2: `bun test tests/honorairesFormat.test.ts` FAIL**

- [ ] **Step 3: Ajouter dans `format.ts` :**

```ts
import type { MoyenPaiement } from './types';

export const MOYEN_PAIEMENT_LABELS: Record<MoyenPaiement, string> = {
	especes: 'Espèces',
	cheque: 'Chèque',
	cb: 'Carte',
	stripe: 'Stripe'
};

export function moyenPaiementLabel(m: MoyenPaiement): string {
	return MOYEN_PAIEMENT_LABELS[m];
}
```

`types.ts` : `export type MoyenPaiement = 'especes' | 'cheque' | 'cb' | 'stripe';` + structs.

`api.ts` : invokes. `honorairesOuvrir` → `invoke<string>('honoraires_ouvrir', { id })`.

`clientsUpsert` ajoute `adresse?: string | null`.

`errors.ts` : si `raw` contient `notes d'honoraires` / `nom du cabinet` / `préfixe` → `return raw` (déjà couvert par la branche `< 120`).

- [ ] **Step 4: `bun test tests/honorairesFormat.test.ts tests/errors.test.ts` et `bun run check`**
Expected: PASS

- [ ] **Step 5: Commit** skippable

---

### Task 11: Réglages Cabinet

**Files:**
- Modify: `src/routes/reglages/+page.svelte`

**Interfaces:**
- Consumes: `SettingsPublic` cabinet, `settingsSet`
- Produces: bloc **Cabinet** en tête du formulaire (avant ntfy)

- [ ] **Step 1: Champs** `cabinetNom`, `cabinetAdresse`, `cabinetTelephone`, `cabinetEmail`, `cabinetSiret`, `mentionTva`, `prefixeNumero` initialisés depuis `settingsGet`. Inclus dans `settingsSet`. Défaut mention = texte spec si vide au load.

Section :

```svelte
<section class="flex max-w-lg flex-col gap-4">
  <h2 class="text-sm font-medium">Cabinet</h2>
  <Label for="cab-nom">Nom</Label>
  <Input id="cab-nom" bind:value={cabinetNom} />
  <!-- adresse, téléphone, email, SIRET, mention TVA, préfixe numéro -->
  <p class="text-muted-foreground text-xs">Vide = 2026-0001. Exemple NH → NH-2026-0001.</p>
</section>
```

Pas d’emoji. Tiret ASCII.

- [ ] **Step 2: `bun run check`**
Expected: PASS

- [ ] **Step 3: Commit** skippable

---

### Task 12: Dialog + page `/honoraires` + sidebar

**Files:**
- Create: `src/lib/components/HonoraireDialog.svelte`
- Create: `src/lib/components/HonoraireContextMenu.svelte`
- Create: `src/routes/honoraires/+page.svelte`
- Modify: `src/lib/components/AppSidebar.svelte`

**Interfaces:**
- Consumes: api honoraires, `ClientCombobox`, `openPath` `@tauri-apps/plugin-opener`
- Produces: génération / liste / ouvrir / annuler

`HonoraireDialog` props :

```ts
open: boolean;
clientId?: string;      // prérempli (fiche)
rdvIds?: string[];      // prérempli (1 RDV)
onClose: () => void;
onSaved: (h: HonoraireDetail) => void;
```

Comportement : si `clientId` fixé, pas de combobox. Load `honorairesRdvsDisponibles`. Checkboxes. Select moyen (défaut `especes`). Submit `honorairesCreate` puis `honorairesOuvrir` + `openPath`. Si erreur nom cabinet : toast + lien `/reglages`.

`HonoraireContextMenu` : Ouvrir ; Annuler si `emise` (dialog confirm « Annuler cette note d'honoraires ? Le numéro est conservé. »).

Page : `PageHeader` titre Honoraires, bouton Nouvelle note, table numéro / date / client / total (`formatCentimes`) / statut Émise|Annulée. Clic ligne = ouvrir. Plus récente en tête (`honorairesList()`).

Sidebar : item `{ title: 'Honoraires', href: '/honoraires', icon: ReceiptIcon }` **juste après Clients**. Icon `@lucide/svelte/icons/receipt`.

- [ ] **Step 1: Créer les 3 fichiers + sidebar**

Ouvrir PDF :

```ts
import { openPath } from '@tauri-apps/plugin-opener';
const path = await honorairesOuvrir(id);
await openPath(path);
```

- [ ] **Step 2: `bun run check`**
Expected: PASS

- [ ] **Step 3: Commit** skippable

---

### Task 13: Fiche client (adresse + bloc notes)

**Files:**
- Modify: `src/lib/components/ClientForm.svelte`
- Modify: `src/routes/clients/[id]/+page.svelte`
- Modifier aussi le dialog de création client (`src/routes/clients/+page.svelte`) si `ClientForm` y est utilisé : passer `bind:adresse`

**Interfaces:**
- Create variant : extra « Adresse » dans « Ajouter des champs »
- Edit variant : champ Adresse dans Identité
- Fiche : bloc liste honoraires + bouton Regrouper (`HonoraireDialog` `clientId={id}`)

- [ ] **Step 1: `ClientForm` + bind `adresse`**

Create : `showAdresse`, checkbox, input textarea ou Input. Edit : Input sous téléphone.

`clients/[id]/+page.svelte` : state `adresse`, load/save via upsert, `honorairesList(clientId)`, table + context menu, dialog regroupement.

- [ ] **Step 2: `bun run check`**
Expected: PASS

- [ ] **Step 3: Commit** skippable

---

### Task 14: Panneau RDV + clic droit

**Files:**
- Modify: `src/lib/components/RdvContextMenu.svelte`
- Modify: `src/lib/components/RdvPanel.svelte`
- Create (si besoin d’un helper) : `src/lib/honoraireRdv.ts`

**Interfaces:**
- `honoraireForRdv(rdvId)` : `honorairesList` filtré côté client **ou** mieux `honoraires_rdvs_disponibles` + get. Pour ne pas lister toute la base : `honorairesList(rdv.client_id)` puis trouver une note `emise` dont une ligne a ce `rdv_id`. Ça exige `honoraires_get` pour les lignes. Alternative YAGNI : commande déjà là. Sur le panneau, `honorairesList(client_id)` + `Promise.all` get est lourd.

Décision YAGNI : étendre `honoraires_list` **n’est pas** nécessaire. Ajouter dans Rust (si pas déjà) un champ optionnel trop tard. À la place : `honoraires_rdvs_disponibles` dit si le RDV est libre. Si libre et `planifie` → Générer. Sinon `honorairesList(client_id)` puis pour chaque note `emise` `honorairesGet` jusqu’à trouver le rdv (N notes d’un client, OK solo).

Helper :

```ts
export async function findHonoraireEmisForRdv(clientId: string, rdvId: string): Promise<Honoraire | null>
```

dans `src/lib/honoraireRdv.ts`.

Menu : après Stripe, « Note d'honoraires » (dialog 1 id) ou « Ouvrir la note ». Panneau : même chose en bouton.

Ne pas proposer sur RDV `annule`.

- [ ] **Step 1: Helper + menu + panneau** (`HonoraireDialog` avec `rdvIds={[rdv.id]}` et `clientId`)

- [ ] **Step 2: `bun run check`**
Expected: PASS

- [ ] **Step 3: `cd src-tauri && cargo test --lib` et `bun test`**
Expected: PASS

- [ ] **Step 4: Commit** skippable

---

### Task 15: Vérif manuelle Debian 13

**Files:** aucune (sauf fix bugs trouvés)

- [ ] **Step 1:** `bun run tauri dev`
- [ ] **Step 2:** Réglages → nom cabinet → Enregistrer
- [ ] **Step 3:** RDV existant → clic droit → Note d'honoraires → Espèces → PDF s’ouvre, fichier `~/Synapt/honoraires/{annee}-0001.pdf`
- [ ] **Step 4:** `/honoraires` liste la note, clic ouvre
- [ ] **Step 5:** Fiche client → adresse optionnelle → Regrouper 2 séances → total = somme
- [ ] **Step 6:** Annuler → bandeau ANNULÉE ; réémettre → numéro suivant, pas de réutilisation
- [ ] **Step 7:** Nom cabinet vidé → génération toaste vers réglages

Critère spec : ces 7 points OK.

---

## Self-review (coverage)

| Spec | Task |
|---|---|
| Numéro / préfixe | 1, 4 |
| Cabinet settings | 1, 11 |
| Tables + adresse client | 2, 13 |
| Create 1 / N / snapshots / PDF dir | 3, 5, 7 |
| Refus métier | 4 |
| Annuler + seq | 6 |
| Ouvrir regen | 7 |
| clients_delete | 8 |
| Commands + opener path | 9 |
| `/honoraires` + dialog + clic droit liste | 12 |
| Fiche + regrouper | 13 |
| RDV panneau / context | 14 |
| Erreurs FR | 4, 10 |
| Tests cargo listés | 3-8 |
| Hors Factur-X / email | (rien à faire) |
