# Synapt MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Sous-agents d’implémentation : **Composer 2.5 normal**, pas fast. Ne pas modifier ce plan pendant l’implémentation ; cocher les `- [ ]` uniquement. Ne pas recréer ces todos. Commits seulement si l’humain l’a demandé dans la session (sinon skip la step commit).

**Goal:** App desktop Linux Tauri 2 utilisable : dashboard, agenda, clients, tarifs, notes, Jitsi, Payment Links Stripe, rappels ntfy téléphone.

**Architecture:** Frontend SvelteKit SPA via `invoke` uniquement. Rust possède SQLite (`~/.local/share/synapt/synapt.db`), `settings.json` (`~/.config/synapt/`, mode 0600), Stripe et ntfy. Aucun SQL ni secret dans le webview.

**Tech Stack:** Tauri 2, SvelteKit 2, Svelte 5, Tailwind 4, shadcn-svelte (new-york), Bits UI, Bun, rusqlite, reqwest, opener, clipboard-manager.

**Spec:** `docs/superpowers/specs/2026-09-11-synapt-mvp-design.md`

## Global Constraints

- Cible Debian 13, app solo locale, pas de cloud.
- SvelteKit SPA : `adapter-static`, `ssr = false`, `prerender = false`, fallback `index.html`.
- Install / scripts JS : Bun uniquement.
- Prix en centimes `i64`, IDs UUID v4 texte, datetimes UTC ISO 8601.
- Frontend : ni SQL, ni Stripe, ni ntfy.
- Pas de facture PDF, pas de plugin-sql, pas de plugin-notification, pas de Radix React, pas de Superforms.
- Sidebar : Dashboard `/`, Agenda `/agenda`, Clients `/clients`, Tarifs `/tarifs`, Notes `/notes`, Réglages `/reglages`.
- Pas de suppression client dans le MVP. Tarif : `actif = 0`, pas de DELETE.
- Commandes Tauri snake_case listées dans la spec.
- UI français, sans emoji.

## File map

```
package.json
svelte.config.js
vite.config.ts
src/app.html
src/app.css
src/routes/+layout.ts
src/routes/+layout.svelte
src/routes/+page.svelte
src/routes/agenda/+page.svelte
src/routes/clients/+page.svelte
src/routes/clients/[id]/+page.svelte
src/routes/tarifs/+page.svelte
src/routes/notes/+page.svelte
src/routes/reglages/+page.svelte
src/lib/api.ts
src/lib/types.ts
src/lib/format.ts
src/lib/components/AppSidebar.svelte
src/lib/components/RdvDialog.svelte
src/lib/components/RdvPanel.svelte
src/lib/components/WeekGrid.svelte
src-tauri/Cargo.toml
src-tauri/src/lib.rs
src-tauri/src/main.rs
src-tauri/src/db.rs
src-tauri/src/models.rs
src-tauri/src/error.rs
src-tauri/src/settings.rs
src-tauri/src/overlap.rs
src-tauri/src/ntfy.rs
src-tauri/src/stripe.rs
src-tauri/src/commands.rs
src-tauri/tauri.conf.json
src-tauri/capabilities/default.json
```

---

### Task 1: Scaffold Tauri 2 + SvelteKit SPA + shadcn

**Files:**
- Create: `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`, `src/app.html`, `src/app.css`, `src/routes/+layout.ts`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `src-tauri/**` via CLI
- Preserve: `AGENTS.md`, `docs/`

**Interfaces:**
- Consumes: rien
- Produces: `bun run tauri dev` lance une fenêtre ; route `/` rend « Synapt »

- [ ] **Step 1: Scaffold SvelteKit à la racine sans écraser docs/AGENTS.md**

Dans `/home/leo/Perso/Projet/Dev/Synapt` :

```bash
bun x sv create . --template minimal --types ts --no-install
```

Si le CLI refuse un dossier non vide, créer les fichiers à la main (contenu ci-dessous) plutôt que d’écraser `docs/` ou `AGENTS.md`.

`package.json` :

```json
{
  "name": "synapt",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "tauri": "tauri"
  }
}
```

```bash
bun add -D svelte @sveltejs/kit @sveltejs/vite-plugin-svelte vite typescript svelte-check @sveltejs/adapter-static tailwindcss @tailwindcss/vite
bun add @tauri-apps/api
bun add -D @tauri-apps/cli
```

`svelte.config.js` :

```js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ fallback: 'index.html' })
  }
};

export default config;
```

`src/routes/+layout.ts` :

```ts
export const ssr = false;
export const prerender = false;
```

`vite.config.ts` : port **1420**, `strictPort: true`, `watch.ignored: ['**/src-tauri/**']`, plugin `sveltekit()` + `tailwindcss()`, `clearScreen: false`.

- [ ] **Step 2: Init Tauri 2**

```bash
bun x tauri init --ci --app-name Synapt --window-title Synapt --dev-url http://localhost:1420 --before-dev-command "bun run dev" --before-build-command "bun run build"
```

Dans `src-tauri/tauri.conf.json` : `frontendDist` = `../build`, `devUrl` = `http://localhost:1420`, identifier `com.synapt.app`.

```bash
bun tauri add opener
bun tauri add clipboard-manager
```

Enregistrer les plugins dans `src-tauri/src/lib.rs`. Permissions `opener:default` et `clipboard-manager:allow-write-text` (+ read si besoin) dans `src-tauri/capabilities/default.json`.

- [ ] **Step 3: shadcn-svelte**

```bash
bun x shadcn-svelte@latest init --base-color neutral --css src/app.css
bun x shadcn-svelte@latest add button sidebar dialog input textarea table sonner calendar dropdown-menu label separator sheet badge
bun add bits-ui @lucide/svelte svelte-sonner mode-watcher clsx tailwind-merge tailwind-variants @internationalized/date
```

`src/routes/+layout.svelte` : `ModeWatcher`, `Toaster` sonner, `{@render children()}`. Page `/` : un `<h1>Synapt</h1>`.

- [ ] **Step 4: Vérifier le build frontend**

Run: `bun run check`

Expected: exit 0 (warnings TS mineurs OK, pas d’erreur adapter/ssr).

Run: `bun run build`

Expected: dossier `build/` avec `index.html`.

- [ ] **Step 5: Commit (si l’humain l’a demandé)**

```bash
git add package.json svelte.config.js vite.config.ts src src-tauri bun.lock
git commit -m "$(cat <<'EOF'
chore: scaffold Tauri 2 SvelteKit SPA et shadcn-svelte

EOF
)"
```

---

### Task 2: Erreurs, modèles, overlap, SQLite

**Files:**
- Create: `src-tauri/src/error.rs`, `src-tauri/src/models.rs`, `src-tauri/src/overlap.rs`, `src-tauri/src/db.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`
- Test: `src-tauri/src/overlap.rs` (`#[cfg(test)]`), `src-tauri/src/db.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: rien
- Produces:
  - `AppError` serializable `{ message: String }`
  - `open_memory() -> Result<Connection>`
  - `open_file(path: &Path) -> Result<Connection>`
  - `migrate(conn: &Connection) -> Result<()>`
  - `fn overlaps(a_start: &str, a_mins: i64, b_start: &str, b_mins: i64) -> Result<bool, AppError>`
  - structs `Client`, `Tarif`, `Rdv`, `Note`, `RappelNtfy` (champs = colonnes spec)

- [ ] **Step 1: Test overlap qui échoue**

Dans `src-tauri/src/overlap.rs` :

```rust
pub fn overlaps(a_start: &str, a_mins: i64, b_start: &str, b_mins: i64) -> Result<bool, crate::error::AppError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chevauchement_simple() {
        assert!(overlaps(
            "2026-09-11T10:00:00Z",
            60,
            "2026-09-11T10:30:00Z",
            30
        ).unwrap());
    }

    #[test]
    fn adjacent_pas_chevauchement() {
        assert!(!overlaps(
            "2026-09-11T10:00:00Z",
            60,
            "2026-09-11T11:00:00Z",
            30
        ).unwrap());
    }
}
```

`Cargo.toml` deps : `rusqlite` (bundled), `serde`, `serde_json`, `uuid`, `chrono`, `thiserror`, `reqwest` (json, rustls-tls), `tokio` (macros, time).

- [ ] **Step 2: Lancer le test (doit fail)**

Run: `cd src-tauri && cargo test overlaps -- --nocapture`

Expected: FAIL (`todo` panic) ou compile error si le module n’est pas dans `lib.rs`.

- [ ] **Step 3: Implémenter overlap + migrate**

`overlaps` : parser RFC3339 avec `chrono::DateTime<Utc>`, intervalle `[start, start + mins)` , vrai ssi `a_start < b_end && b_start < a_end`.

`migrate` exécute exactement :

```sql
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
```

Chemin prod : `dirs::data_dir()/synapt/synapt.db` (`~/.local/share/synapt/` sous Linux). Crate `directories` ou `dirs`. Créer le dossier.

Test `db` : `open_memory` + `migrate` + `SELECT name FROM sqlite_master` contient `rdv`.

- [ ] **Step 4: Tests passent**

Run: `cd src-tauri && cargo test`

Expected: PASS (overlap + migrate).

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src-tauri
git commit -m "$(cat <<'EOF'
feat: schéma SQLite et détection de chevauchement RDV

EOF
)"
```

---

### Task 3: CRUD métier RDV (tests spec, sans réseau)

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/db.rs`, `src-tauri/src/models.rs`, `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands.rs` `#[cfg(test)]`

**Interfaces:**
- Consumes: `migrate`, `overlaps`, `AppError`
- Produces (signatures Rust, plus tard exposées en commands Tauri) :
  - `fn jitsi_url(id: &str) -> String` → `https://meet.jit.si/synapt-{id}`
  - `fn rdv_create(conn, client_id, tarif_id: Option<String>, debut, duree_minutes, note) -> Result<Rdv, AppError>`
  - `fn rdv_annuler(conn, id) -> Result<Rdv, AppError>`
  - `fn rdv_list(conn, from: &str, to: &str) -> Result<Vec<Rdv>, AppError>` (statut `planifie` only pour l’agenda ; `rdv_get` peut renvoyer un annulé)
  - `fn rdv_dashboard(conn, now: DateTime<Utc>) -> Result<Dashboard, AppError>`
  - `struct Dashboard { aujourdhui: Vec<Rdv>, a_venir: Vec<Rdv> }` (`a_venir` max 5, après la fin du jour local… **spécifier UTC jour calendaire de `now`** : aujourd’hui = `[now.date(), now.date()+1day)` en UTC pour le MVP, documenté ainsi pour éviter l’ambiguïté TZ)
  - `fn clients_upsert`, `fn clients_list`, `fn clients_get`
  - `fn tarifs_upsert`, `fn tarifs_list`, `fn tarifs_set_actif`
  - `fn notes_upsert`, `fn notes_list(client_id: Option<String>, perso: bool)`, `fn notes_delete`

Décision TZ dashboard (figée) : filtrer avec le fuseau **local de la machine** (`Local::now().date_naive()`), pas UTC brut. Les instants en base restent UTC.

- [ ] **Step 1: Tests spec qui échouent**

```rust
#[test]
fn create_rdv_jitsi_et_planifie() {
    let conn = crate::db::open_memory().unwrap();
    // insert client + tarif
    let rdv = rdv_create(&conn, &client.id, Some(tarif.id.clone()), "2026-09-11T10:00:00Z", 60, None).unwrap();
    assert!(rdv.jitsi_url.starts_with("https://meet.jit.si/synapt-"));
    assert_eq!(rdv.statut, "planifie");
}

#[test]
fn conflit_horaire_refuse() {
    let conn = crate::db::open_memory().unwrap();
    rdv_create(&conn, &c, Some(t.clone()), "2026-09-11T10:00:00Z", 60, None).unwrap();
    let err = rdv_create(&conn, &c, Some(t), "2026-09-11T10:30:00Z", 30, None).unwrap_err();
    assert!(err.message.contains("chevauche") || err.message.contains("horaire"));
}

#[test]
fn annuler_libere_le_creneau() {
    let conn = crate::db::open_memory().unwrap();
    let a = rdv_create(&conn, &c, Some(t.clone()), "2026-09-11T10:00:00Z", 60, None).unwrap();
    rdv_annuler(&conn, &a.id).unwrap();
    let b = rdv_create(&conn, &c, Some(t), "2026-09-11T10:00:00Z", 60, None).unwrap();
    assert_eq!(b.statut, "planifie");
}
```

Helpers de test : fonctions `seed_client` / `seed_tarif` dans le même module test.

`rdv_create` ne doit **pas** appeler Stripe/ntfy (hooks `Option` plus tard). Conflit : uniquement vs `statut = 'planifie'`.

`jitsi_url` immuable : `rdv_update` ne change pas `jitsi_url`.

- [ ] **Step 2: Run tests fail**

Run: `cd src-tauri && cargo test rdv_ -- --nocapture`

Expected: FAIL (fonctions absentes).

- [ ] **Step 3: Implémenter CRUD**

`rdv_create` : uuid, `jitsi_url = format!("https://meet.jit.si/synapt-{id}")`, `statut = planifie`, charger les `planifie` existants, `overlaps` chacun, insert.

`rdv_annuler` : `UPDATE rdv SET statut='annule'`. (DELETE ntfy dans Task 5.)

`rdv_list` : `WHERE debut >= ? AND debut < ? AND statut = 'planifie' ORDER BY debut`.

Pas de `DELETE FROM clients`.

- [ ] **Step 4: Tests passent**

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src-tauri/src
git commit -m "$(cat <<'EOF'
feat: CRUD RDV avec Jitsi et refus des chevauchements

EOF
)"
```

---

### Task 4: Settings + ntfy planif (pures) + Stripe HTTP

**Files:**
- Create: `src-tauri/src/settings.rs`, `src-tauri/src/ntfy.rs`, `src-tauri/src/stripe.rs`
- Test: `src-tauri/src/ntfy.rs`, `src-tauri/src/settings.rs`

**Interfaces:**
- Consumes: `Rdv`, `AppError`
- Produces:
  - `struct Settings { ntfy: NtfySettings, stripe: StripeSettings }`
  - `NtfySettings { serveur, topic, token, rappel_24h, rappel_1h }`
  - `StripeSettings { secret_key }`
  - `struct SettingsPublic { ntfy_serveur, ntfy_topic, ntfy_token_configured: bool, ntfy_token_last4: String, rappel_24h, rappel_1h, stripe_configured: bool, stripe_last4: String }`
  - `fn settings_path() -> PathBuf` → `dirs::config_dir()/synapt/settings.json`
  - `fn load_settings() -> Settings` (fichier manquant ou JSON cassé → défauts spec, pas de panic)
  - `fn save_settings(s: &Settings) -> Result<()>` (mkdir, write, `0o600`)
  - `fn mask_secret(s: &str) -> (bool, String)` configured + last4
  - `enum RappelKind { H24, H1 }` `as_str() -> "24h" | "1h"`
  - `fn echeance(debut: DateTime<Utc>, kind: RappelKind) -> DateTime<Utc>` → `debut - 24h` / `debut - 1h`
  - `fn dans_fenetre_ntfy(now, echeance) -> bool` : `echeance > now` et `echeance - now <= 3 days`
  - `fn should_publish(topic: &str, flag: bool, statut: &str, now, echeance, deja_programme: bool) -> bool`
  - `async fn ntfy_publish(settings, title, message, click, delay: Option<DateTime<Utc>>, priority: u8) -> Result<String /* ntfy id */>`
  - `async fn ntfy_delete(settings, ntfy_id: &str) -> Result<()>`
  - `async fn ntfy_test(settings) -> Result<()>` body `Synapt OK`
  - `async fn stripe_create_payment_link(secret, nom: &str, prix_centimes: i64) -> Result<(String /*id*/, String /*url*/)>`
  - `const NTFY_MAX: Duration = Duration::days(3);`

- [ ] **Step 1: Tests pures ntfy + mask**

```rust
#[test]
fn echeance_24h() {
    let d = chrono::DateTime::parse_from_rfc3339("2026-09-12T10:00:00Z").unwrap().with_timezone(&chrono::Utc);
    assert_eq!(echeance(d, RappelKind::H24).to_rfc3339(), "2026-09-11T10:00:00+00:00");
}

#[test]
fn fenetre_4_jours_false() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z").unwrap().with_timezone(&chrono::Utc);
    let e = now + chrono::Duration::days(4);
    assert!(!dans_fenetre_ntfy(now, e));
}

#[test]
fn should_publish_sans_topic() {
    assert!(!should_publish("", true, "planifie", now, e, false));
}

#[test]
fn mask_last4() {
    let (ok, last) = mask_secret("sk_test_abcdefghij");
    assert!(ok);
    assert_eq!(last, "ghij");
}
```

`ntfy_publish` / `stripe_create_payment_link` : **aucun appel réseau dans ces tests**.

- [ ] **Step 2: Run fail**

Run: `cd src-tauri && cargo test dans_fenetre -- --nocapture`

Expected: FAIL.

- [ ] **Step 3: Implémenter settings + HTTP**

`load_settings` défauts :

```json
{
  "ntfy": {
    "serveur": "https://ntfy.sh",
    "topic": "",
    "token": "",
    "rappel_24h": true,
    "rappel_1h": true
  },
  "stripe": { "secret_key": "" }
}
```

`ntfy_publish` : `POST {serveur}` JSON `{"topic","title","message","click","delay": <unix si Some>, "priority"}`. Header `Authorization: Bearer {token}` si token non vide. Parser `id` du JSON réponse.

`ntfy_delete` : `DELETE {serveur}/{topic}/{ntfy_id}`.

`stripe_create_payment_link` : `POST https://api.stripe.com/v1/payment_links` `application/x-www-form-urlencoded`, basic auth `secret_key` + mot de passe vide, champs :

```
line_items[0][quantity]=1
line_items[0][price_data][currency]=eur
line_items[0][price_data][unit_amount]={prix_centimes}
line_items[0][price_data][product_data][name]={nom}
```

Retourner `id` + `url`. Si `prix_centimes == 0` : `Err` métier « pas de lien ».

- [ ] **Step 4: Tests pures PASS**

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src-tauri/src
git commit -m "$(cat <<'EOF'
feat: settings locaux, planification ntfy et Payment Links Stripe

EOF
)"
```

---

### Task 5: Commands Tauri + hooks Stripe/ntfy + timer

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/db.rs`
- Test: étendre tests `rdv_create` (hooks no-op)

**Interfaces:**
- Consumes: toutes les fn Task 3–4
- Produces: commands Tauri (noms spec) + `fn ntfy_sync(conn, settings, now) -> Result<Vec<String>>` interne
- `rdv_create` Tauri : insert puis **best-effort** Stripe si `secret_key` non vide et tarif `prix_centimes > 0` ; **best-effort** ntfy pour 24h/1h si `should_publish` ; échec → toast côté UI via `Result` partielle : **le RDV est Ok**, champs stripe/rappels éventuellement vides ; la command retourne `Rdv` quand même. Erreurs Stripe/ntfy : log `eprintln!` ; exposer `warnings: Vec<String>` sur `RdvCreateResult { rdv: Rdv, warnings: Vec<String> }`.

**Contrat frontend figé :**

```ts
type RdvCreateResult = { rdv: Rdv; warnings: string[] };
```

`rdv_create` invoke retourne `RdvCreateResult`. Les autres commands `rdv_*` retournent `Rdv` / `Rdv[]` / `Dashboard`.

`settings_get` → `SettingsPublic`. `settings_set` accepte le formulaire : champs secrets vides = **ne pas écraser** la valeur déjà stockée.

`stripe_ensure_link(rdv_id)` : si `stripe_url` déjà là, le renvoyer ; sinon créer, `UPDATE rdv SET stripe_url, stripe_id`.

`ntfy_sync` : pour chaque rdv `planifie`, chaque kind dont le flag est true, si `should_publish` et pas de row `programme`, `ntfy_publish` avec `delay = echeance`, insert `rappels_ntfy`.

Au `setup` : `migrate` fichier prod, `ntfy_sync`, puis `std::thread` + `std::thread::sleep(Duration::from_secs(3600))` loop (ou `tokio::time::interval` si runtime async Tauri). Ne pas bloquer `setup`.

`rdv_annuler` : pour chaque rappel `programme`, `ntfy_delete` (ignorer l’erreur HTTP après toast/warning), `etat = annule`, puis statut rdv.

- [ ] **Step 1: Test sync ne publie pas hors fenêtre (pure, mock)**

Tester `should_publish` déjà fait. Ajouter test `ntfy_sync` avec un trait `NtfyClient` :

```rust
pub trait NtfyClient {
    fn publish(&self, delay: DateTime<Utc>, kind: RappelKind) -> Result<String, AppError>;
}
```

Impl prod `ReqwestNtfy`. En test, `FakeNtfy { published: Mutex<Vec<RappelKind>> }` : un RDV dans 10 jours → `published` vide ; un RDV dans 2 h avec `rappel_1h` → une pub `H1`.

- [ ] **Step 2: Run fail**

Run: `cd src-tauri && cargo test ntfy_sync -- --nocapture`

Expected: FAIL.

- [ ] **Step 3: Implémenter commands + `invoke_handler`**

`lib.rs` :

```rust
.invoke_handler(tauri::generate_handler![
  settings_get, settings_set,
  clients_list, clients_get, clients_upsert,
  tarifs_list, tarifs_upsert, tarifs_set_actif,
  notes_list, notes_upsert, notes_delete,
  rdv_list, rdv_get, rdv_create, rdv_update, rdv_annuler, rdv_dashboard,
  stripe_ensure_link, ntfy_test
])
```

`rdv_update` : peut changer client/tarif/debut/duree/note ; **interdit** de changer `jitsi_url` ; re-check overlap en excluant `id` courant ; si `debut` change, marquer anciens rappels `annule` + `ntfy_delete`, puis laisser `ntfy_sync` / create reprogrammer.

State Tauri : `Mutex<Connection>` **ou** ouvrir une connexion par command (rusqlite n’est pas Sync). **Décision figée : ouvrir `Connection` par command** via `open_file(&db_path())` (simple, pas de mutex). `db_path()` partagé.

- [ ] **Step 4: cargo test + cargo check**

Run: `cd src-tauri && cargo test && cargo check`

Expected: PASS / Finished.

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src-tauri
git commit -m "$(cat <<'EOF'
feat: commands Tauri, Stripe best-effort et sync ntfy

EOF
)"
```

---

### Task 6: API frontend + coquille sidebar

**Files:**
- Create: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/format.ts`, `src/lib/components/AppSidebar.svelte`
- Modify: `src/routes/+layout.svelte`, `src/routes/+page.svelte`
- Create empty routes: `src/routes/agenda/+page.svelte`, `clients/+page.svelte`, `clients/[id]/+page.svelte`, `tarifs/+page.svelte`, `notes/+page.svelte`, `reglages/+page.svelte`

**Interfaces:**
- Consumes: noms de commands Task 5
- Produces: `src/lib/api.ts` fonctions typées `clientsList()`, `rdvCreate(input)`, etc. ; `formatCentimes(n: number) => string` (`12,50 €`) ; `formatDateTime(iso: string)` via `Intl` `fr-FR`

`types.ts` :

```ts
export type Client = {
  id: string;
  nom: string;
  email: string | null;
  telephone: string | null;
  created_at: string;
  updated_at: string;
};

export type Tarif = {
  id: string;
  nom: string;
  duree_minutes: number;
  prix_centimes: number;
  actif: number;
  created_at: string;
  updated_at: string;
};

export type Rdv = {
  id: string;
  client_id: string;
  tarif_id: string | null;
  debut: string;
  duree_minutes: number;
  jitsi_url: string;
  stripe_url: string | null;
  stripe_id: string | null;
  note: string | null;
  statut: 'planifie' | 'annule';
  created_at: string;
  updated_at: string;
  client_nom?: string;
  tarif_nom?: string;
};

export type RdvCreateResult = { rdv: Rdv; warnings: string[] };

export type Dashboard = { aujourdhui: Rdv[]; a_venir: Rdv[] };

export type SettingsPublic = {
  ntfy_serveur: string;
  ntfy_topic: string;
  ntfy_token_configured: boolean;
  ntfy_token_last4: string;
  rappel_24h: boolean;
  rappel_1h: boolean;
  stripe_configured: boolean;
  stripe_last4: string;
};
```

Si le Rust n’envoie pas `client_nom`, joindre côté UI via `clients_get` **ou** enrichir `rdv_list` / dashboard en SQL `JOIN`. **Décision figée : JOIN dans `rdv_list`, `rdv_get`, `rdv_dashboard`** pour `client_nom` et `tarif_nom`. Ajouter les deux champs Option au struct `Rdv` serde `skip_serializing_if` inutile, toujours présents (string vide si pas de tarif).

- [ ] **Step 1: Enrichir SQL JOIN (Rust) si pas déjà fait**

`SELECT rdv.*, clients.nom as client_nom, tarifs.nom as tarif_nom FROM rdv JOIN clients ... LEFT JOIN tarifs`. Adapter les tests Task 3 : `client_nom` égal au nom seedé.

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 2: `api.ts`**

```ts
import { invoke } from '@tauri-apps/api/core';
import type { Client, Dashboard, Note, Rdv, RdvCreateResult, SettingsPublic, Tarif } from './types';

export const rdvDashboard = () => invoke<Dashboard>('rdv_dashboard');
export const rdvCreate = (p: {
  client_id: string;
  tarif_id: string | null;
  debut: string;
  duree_minutes: number;
  note: string | null;
}) => invoke<RdvCreateResult>('rdv_create', p);
```

Même schéma pour toutes les commands (un invoke par nom spec). `notesList({ client_id?: string; perso?: boolean })`.

- [ ] **Step 3: Sidebar + layout**

`AppSidebar.svelte` : liens `/`, `/agenda`, `/clients`, `/tarifs`, `/notes`, `/reglages`. Labels français.

`+layout.svelte` : `SidebarProvider` + `AppSidebar` + `SidebarInset` + children + `Toaster`.

Pages placeholder : titre de la section.

- [ ] **Step 4: `bun run check`**

Expected: exit 0.

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src src-tauri
git commit -m "$(cat <<'EOF'
feat: API invoke et navigation sidebar

EOF
)"
```

---

### Task 7: Réglages, clients, tarifs, notes UI

**Files:**
- Modify: `src/routes/reglages/+page.svelte`, `src/routes/clients/+page.svelte`, `src/routes/clients/[id]/+page.svelte`, `src/routes/tarifs/+page.svelte`, `src/routes/notes/+page.svelte`
- Create: `src/lib/components/RdvDialog.svelte` (utilisé aussi Task 8)

**Interfaces:**
- Consumes: `api.ts`, `SettingsPublic`
- Produces: écrans CRUD utilisables ; `RdvDialog` props :

```ts
{
  open: boolean;
  presetDebut?: string; // ISO UTC
  presetClientId?: string;
  onClose: () => void;
  onSaved: (r: RdvCreateResult) => void;
}
```

- [ ] **Step 1: Page réglages**

Charger `settings_get`. Formulaire contrôlé `$state`. Sauvegarde `settings_set` : `secret_key` et `token` envoyés seulement si l’utilisateur a tapé une nouvelle valeur (sinon `""` = keep). Bouton Tester → `ntfy_test` → `toast.success` / `toast.error`. Clés affichées `••••ghij` si configured.

- [ ] **Step 2: Tarifs**

Table shadcn. Dialog créer/éditer : nom, durée minutes, prix en euros saisi (`12.5` → `1250` centimes). Bouton Désactiver → `tarifs_set_actif(id, 0)`. Liste création RDV : `actif === 1` only.

- [ ] **Step 3: Clients + fiche**

Liste + input recherche filtre `nom` client-side. Clic → `/clients/{id}`. Fiche : champs + `notes_list({ client_id })` + historique `rdv_list` large plage **ou** nouvelle command. Pour l’historique : `rdv_list` from `1970` to `2100` puis `filter client_id` est trop large. **Décision : `rdv_list` accepte `client_id` optionnel.** Ajouter `client_id: Option<String>` au payload `rdv_list`. Si `Some`, ignorer from/to optionnels ou les AND. Implémenter en Rust + test : deux clients, list filtrée n’en renvoie qu’un. **Pas de bouton supprimer.**

- [ ] **Step 4: Notes perso + RdvDialog**

`/notes` : `notes_list({ perso: true })`, textarea + enregistrer, supprimer `notes_delete`.

`RdvDialog` : select client, select tarif actif (on change : `duree_minutes = tarif.duree_minutes`), datetime-local converti en UTC ISO, note. Submit `rdvCreate`. Afficher `warnings` en toast. Erreur chevauchement : texte sous le form, pas de close.

- [ ] **Step 5: `bun run check` + `cd src-tauri && cargo test`**

Expected: PASS.

- [ ] **Step 6: Commit (si demandé)**

```bash
git add src src-tauri
git commit -m "$(cat <<'EOF'
feat: écrans réglages, clients, tarifs, notes et dialog RDV

EOF
)"
```

---

### Task 8: Dashboard + agenda + panneau RDV

**Files:**
- Modify: `src/routes/+page.svelte`, `src/routes/agenda/+page.svelte`
- Create: `src/lib/components/WeekGrid.svelte`, `src/lib/components/RdvPanel.svelte`

**Interfaces:**
- Consumes: `rdvDashboard`, `rdvList`, `RdvDialog`, `Rdv`
- Produces: dashboard spec ; grille semaine 8h–20h pas 30 min ; panneau sheet

- [ ] **Step 1: Dashboard**

`onMount` → `rdvDashboard` + `settingsGet`. Bandeau si `!ntfy_topic` (topic vide) **ou** `!stripe_configured` : texte + lien `/reglages`. Liste aujourd’hui : heure locale, nom, tarif. Boutons : ouvrir Jitsi (`openUrl` `@tauri-apps/plugin-opener`), copier Jitsi (`writeText` clipboard-manager), Stripe si `stripe_url`, sinon masquer. Lien fiche `/clients/{id}`. Bloc à venir (5). Bouton Nouveau RDV ouvre `RdvDialog`. Lien « Agenda ».

- [ ] **Step 2: WeekGrid**

Props : `startMonday: Date` (local), `rdvs: Rdv[]`, `onSlot(isoUtc: string)`, `onRdv(r: Rdv)`. Colonnes lun–dim. Lignes 08:00–20:00 step 30 min. RDV positionnés en `top/height` % de la journée affichée. Clic fond → `onSlot`. Clic event → `onRdv`. Vue jour : une colonne. Toggle semaine/jour dans `/agenda`. Date picker Bits Calendar pour changer de semaine.

Filtrer `rdv_list({ from, to })` UTC correspondant à la semaine locale affichée.

- [ ] **Step 3: RdvPanel**

`Sheet` : client, horaire, note, état rappels (`notes` : si pas d’API dédiée, afficher Jitsi/Stripe seulement ; **ajouter `rappels` sur `rdv_get`**).

**Décision :** `rdv_get` retourne `RdvDetail { rdv: Rdv, rappels: RappelNtfy[] }`.

```ts
export type RappelNtfy = {
  id: string;
  rdv_id: string;
  type: '24h' | '1h';
  ntfy_id: string | null;
  echeance: string;
  etat: 'programme' | 'annule';
};
```

Boutons : copier/ouvrir Jitsi ; `stripe_ensure_link` puis copier/ouvrir ; modifier (réouvre dialog prérempli via `rdv_update` : étendre `RdvDialog` mode edit `rdvId?: string` → `invoke('rdv_update')`) ; Annuler → `rdv_annuler` + fermer.

`rdv_update` payload : `{ id, client_id, tarif_id, debut, duree_minutes, note }`.

- [ ] **Step 4: Tests Rust rdv_get rappels + bun check**

Test : create rdv sans ntfy → `rappels` vide. `bun run check`. `cd src-tauri && cargo test`.

Expected: PASS.

- [ ] **Step 5: Commit (si demandé)**

```bash
git add src src-tauri
git commit -m "$(cat <<'EOF'
feat: dashboard, grille agenda et panneau RDV

EOF
)"
```

---

### Task 9: Branchement final + critère MVP

**Files:**
- Modify: `src/lib/components/RdvDialog.svelte`, `src/routes/+page.svelte`, `src/routes/agenda/+page.svelte` (toasts warnings)
- Modify: `src-tauri/src/lib.rs` si timer manquant

**Interfaces:**
- Consumes: tout
- Produces: `bun run tauri dev` satisfait le critère spec

- [ ] **Step 1: Timer ntfy_sync**

Dans `setup` de `lib.rs`, `tauri::async_runtime::spawn` loop 1 h + un appel immédiat. Catch errors `eprintln`.

- [ ] **Step 2: Toasts warnings**

Chaque `RdvCreateResult.warnings` → `toast.error` par entrée. Échec invoke → `toast.error(e)`.

- [ ] **Step 3: Vérif manuelle (Debian)**

Run: `bun run tauri dev`

Expected: fenêtre Synapt. Checklist :

1. Créer un tarif et un client
2. Créer un RDV depuis le dashboard → Jitsi copiable / ouvrable dans le navigateur
3. Conflit : second RDV qui chevauche → message, pas de création
4. Note perso + note sur fiche client
5. Réglages : sans topic/clé, app OK, bandeau dashboard
6. Agenda semaine : RDV visible, panneau, annuler, recréer le créneau

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 4: Commit (si demandé)**

```bash
git add src src-tauri
git commit -m "$(cat <<'EOF'
feat: rappels ntfy en fond et MVP desktop utilisable

EOF
)"
```

---

## Couverture spec

| Spec | Task |
|---|---|
| Stack Tauri/SvelteKit/shadcn/Bun | 1 |
| SQLite XDG + schéma | 2 |
| Overlap + Jitsi + tests cargo | 2–3 |
| Clients/tarifs/notes CRUD, pas de delete client | 3, 7 |
| Dashboard + bandeau | 8 |
| Agenda grille maison | 8 |
| Settings 0600 + mask | 4–5, 7 |
| ntfy Delay 3 jours + sync 1 h + test + DELETE | 4–5, 9 |
| Stripe Payment Link best-effort | 4–5, 8 |
| opener + clipboard | 1, 8 |
| Hors périmètre respecté | global |

## Self-review

- Pas de TBD. Décisions figées : connexion rusqlite par command ; dashboard en date locale ; JOIN noms ; `rdv_list.client_id` ; `RdvCreateResult.warnings` ; `rdv_get` → `RdvDetail` ; secrets vides = keep.
- Types alignés `Rdv`, `SettingsPublic`, `RappelKind` `"24h"|"1h"` partout.
- `rdv_update` ajouté (spec le listait) : Jitsi immuable, overlap hors soi, reprogram ntfy.
