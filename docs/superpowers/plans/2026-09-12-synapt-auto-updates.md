# Synapt auto-updates `.deb` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** L’app Debian notifie une mise à jour GitHub privée, installe le `.deb` après confirmation (Polkit), puis relance.

**Architecture:** Plugin updater (header Bearer compilé via `SYNAPT_UPDATER_TOKEN`) + plugin process. Check prod au démarrage, bandeau layout, `dpkg -i` via le plugin. CI tag `v*` publie le `.deb` signé et `latest.json`.

**Tech Stack:** Tauri 2, `tauri-plugin-updater` ≥ 2.10.1, `tauri-plugin-process`, Svelte 5, Bun, GitHub Actions (`tauri-action`), Debian `.deb`.

**Spec:** `docs/superpowers/specs/2026-09-12-synapt-auto-updates-design.md`

## Global Constraints

- Cible Linux Debian, bundle updater = `.deb` uniquement (`--bundles deb` en CI)
- Token PAT uniquement via `option_env!("SYNAPT_UPDATER_TOKEN")` côté Rust, jamais dans le JS ni git
- Endpoint : `https://github.com/sylecium/Synapt/releases/latest/download/latest.json`
- `createUpdaterArtifacts: true`, pubkey minisign dans `tauri.conf.json` (contenu, pas un chemin)
- Check au démarrage : silencieux si `import.meta.env.DEV`, déjà à jour, ou échec
- Toasts FR seulement après Installer : téléchargement / accès refusé / pas fiable / installation annulée
- Capability réelle pour relancer : `process:allow-restart` (commande Tauri `restart`, API JS `relaunch`)
- Bun pour JS. Ne pas toucher `docs/superpowers/plans/2026-09-11-synapt-mvp.md` ni `features = []` bruyant dans `Cargo.toml`
- Commits FR conventional, un par tâche. Ne pas committer de clé privée ni de PAT

## File map

- Create: `src/lib/updater-errors.ts`, `src/lib/updater-errors.test.ts`, `src/lib/updater.ts`, `src/lib/components/UpdateBanner.svelte`, `.github/workflows/release.yml`
- Modify: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`, `src/routes/+layout.svelte`

---

### Task 1: Plugins, config, header Rust, pubkey

**Files:**
- Modify: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` (via cargo), `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`
- Test: `bun run check` ; `cargo check` dans `src-tauri`

**Interfaces:**
- Consumes: `tauri::Builder` actuel dans `src-tauri/src/lib.rs`
- Produces: plugin updater avec header Bearer si `SYNAPT_UPDATER_TOKEN` ; plugin process ; `plugins.updater` + `createUpdaterArtifacts` ; permissions `updater:default` et `process:allow-restart`

- [ ] **Step 1: Dépendances JS**

```bash
bun add @tauri-apps/plugin-updater @tauri-apps/plugin-process
```

- [ ] **Step 2: Dépendances Rust**

Dans `src-tauri/Cargo.toml`, ajouter sous `[dependencies]` (sans toucher `tauri` `features = []`) :

```toml
tauri-plugin-updater = "2.10.1"
tauri-plugin-process = "2"
```

Puis :

```bash
cd src-tauri && cargo check
```

Expected: compile OK.

- [ ] **Step 3: Générer la pubkey (clé privée hors repo)**

```bash
mkdir -p "$HOME/.tauri"
if [ ! -f "$HOME/.tauri/synapt.key" ]; then
  bunx tauri signer generate -w "$HOME/.tauri/synapt.key" --password ''
fi
cat "$HOME/.tauri/synapt.key.pub"
```

Copier **uniquement** le contenu de `synapt.key.pub` dans `tauri.conf.json`. Ne pas ajouter `~/.tauri/` au git.

- [ ] **Step 4: `src-tauri/tauri.conf.json`**

Garder le reste du fichier. Ajouter `createUpdaterArtifacts` dans `bundle` et la section `plugins` (pubkey = contenu réel du `.pub`) :

```json
  "bundle": {
    "active": true,
    "createUpdaterArtifacts": true,
    "targets": "all",
```

```json
  "plugins": {
    "updater": {
      "pubkey": "<contenu de synapt.key.pub>",
      "endpoints": [
        "https://github.com/sylecium/Synapt/releases/latest/download/latest.json"
      ]
    }
  }
```

- [ ] **Step 5: Capabilities**

`src-tauri/capabilities/default.json` : ajouter `"updater:default"` et `"process:allow-restart"` au tableau `permissions` (conserver `core:default`, `opener:default`, `clipboard-manager:allow-write-text`).

- [ ] **Step 6: Enregistrer les plugins**

Dans `src-tauri/src/lib.rs`, ajouter avant `fn run` :

```rust
fn updater_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let mut builder = tauri_plugin_updater::Builder::new();
    if let Some(token) = option_env!("SYNAPT_UPDATER_TOKEN") {
        builder = builder
            .header("Authorization", format!("Bearer {token}"))
            .expect("en-tête Authorization updater");
    }
    builder.build()
}
```

Dans `run()`, après `tauri_plugin_opener::init()` :

```rust
        .plugin(updater_plugin())
        .plugin(tauri_plugin_process::init())
```

- [ ] **Step 7: Vérifier**

```bash
cd src-tauri && cargo check
bun run check
```

Expected: 0 erreur. `SYNAPT_UPDATER_TOKEN` absent → header non posé.

- [ ] **Step 8: Commit**

```bash
git add package.json bun.lock src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "$(cat <<'EOF'
feat: brancher l’updater Tauri et le token de release

EOF
)"
```

---

### Task 2: Messages d’erreur d’install (TDD)

**Files:**
- Create: `src/lib/updater-errors.ts`, `src/lib/updater-errors.test.ts`

**Interfaces:**
- Consumes: `unknown` (erreur plugin updater / process)
- Produces: `installUpdateMessage(err: unknown): string` avec exactement les 4 libellés spec + fallback téléchargement

- [ ] **Step 1: Test qui échoue**

Créer `src/lib/updater-errors.test.ts` :

```ts
import { expect, test } from 'bun:test';
import { installUpdateMessage } from './updater-errors';

test('accès GitHub refusé', () => {
	expect(installUpdateMessage('401 Unauthorized')).toBe('Accès aux mises à jour refusé.');
	expect(installUpdateMessage('status code 403')).toBe('Accès aux mises à jour refusé.');
});

test('signature invalide', () => {
	expect(installUpdateMessage('signature verification failed')).toBe(
		'La mise à jour n’est pas fiable, installation annulée.'
	);
	expect(installUpdateMessage('invalid updater binary format')).toBe(
		'La mise à jour n’est pas fiable, installation annulée.'
	);
});

test('Polkit ou dpkg', () => {
	expect(installUpdateMessage('PackageInstallFailed')).toBe(
		'Installation de la mise à jour annulée.'
	);
	expect(installUpdateMessage('pkexec cancelled')).toBe(
		'Installation de la mise à jour annulée.'
	);
});

test('réseau par défaut', () => {
	expect(installUpdateMessage(new Error('Failed to fetch'))).toBe(
		'Impossible de télécharger la mise à jour.'
	);
	expect(installUpdateMessage('')).toBe('Impossible de télécharger la mise à jour.');
});
```

- [ ] **Step 2: Lancer le test (échec)**

```bash
bun test src/lib/updater-errors.test.ts
```

Expected: FAIL (module introuvable).

- [ ] **Step 3: Implémenter**

Créer `src/lib/updater-errors.ts` :

```ts
function rawMessage(err: unknown): string {
	if (typeof err === 'string') return err;
	if (err instanceof Error) return err.message;
	if (err && typeof err === 'object' && 'message' in err) {
		return String((err as { message: unknown }).message);
	}
	return '';
}

export function installUpdateMessage(err: unknown): string {
	const lower = rawMessage(err).toLowerCase();
	if (
		lower.includes('401') ||
		lower.includes('403') ||
		lower.includes('unauthorized') ||
		lower.includes('forbidden')
	) {
		return 'Accès aux mises à jour refusé.';
	}
	if (
		lower.includes('signature') ||
		lower.includes('invalid updater') ||
		lower.includes('not a valid deb')
	) {
		return 'La mise à jour n’est pas fiable, installation annulée.';
	}
	if (
		lower.includes('packageinstallfailed') ||
		lower.includes('pkexec') ||
		lower.includes('dpkg') ||
		lower.includes('cancelled') ||
		lower.includes('canceled')
	) {
		return 'Installation de la mise à jour annulée.';
	}
	return 'Impossible de télécharger la mise à jour.';
}
```

- [ ] **Step 4: Relancer le test**

```bash
bun test src/lib/updater-errors.test.ts
```

Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add src/lib/updater-errors.ts src/lib/updater-errors.test.ts
git commit -m "$(cat <<'EOF'
feat: messages français pour l’échec d’install updater

EOF
)"
```

---

### Task 3: Check prod, bandeau, install + relance

**Files:**
- Create: `src/lib/updater.ts`, `src/lib/components/UpdateBanner.svelte`
- Modify: `src/routes/+layout.svelte`

**Interfaces:**
- Consumes: `check` / `Update` de `@tauri-apps/plugin-updater` ; `relaunch` de `@tauri-apps/plugin-process` ; `installUpdateMessage`
- Produces: `probeUpdate(): Promise<Update | null>` (no-op si `import.meta.env.DEV`) ; bandeau Installer / Plus tard / progression

- [ ] **Step 1: `src/lib/updater.ts`**

```ts
import { check, type Update } from '@tauri-apps/plugin-updater';

export async function probeUpdate(): Promise<Update | null> {
	if (import.meta.env.DEV) return null;
	try {
		const update = await check();
		return update ?? null;
	} catch {
		return null;
	}
}
```

- [ ] **Step 2: `src/lib/components/UpdateBanner.svelte`**

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { relaunch } from '@tauri-apps/plugin-process';
	import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';
	import { Button } from '$lib/components/ui/button/index.js';
	import { installUpdateMessage } from '$lib/updater-errors';
	import { probeUpdate } from '$lib/updater';

	let update = $state<Update | null>(null);
	let dismissed = $state(false);
	let installing = $state(false);
	let percent = $state<number | null>(null);

	onMount(() => {
		void probeUpdate().then((found) => {
			update = found;
		});
	});

	function onDownloadEvent(event: DownloadEvent) {
		switch (event.event) {
			case 'Started': {
				percent = 0;
				break;
			}
			case 'Progress': {
				break;
			}
			case 'Finished': {
				percent = 100;
				break;
			}
			default: {
				const _exhaustive: never = event;
				void _exhaustive;
			}
		}
	}

	async function install() {
		if (!update || installing) return;
		installing = true;
		try {
			let downloaded = 0;
			await update.downloadAndInstall((event) => {
				onDownloadEvent(event);
				if (event.event === 'Started') {
					downloaded = 0;
				}
				if (event.event === 'Progress') {
					downloaded += event.data.chunkLength;
					const total = event.data.contentLength;
					if (total && total > 0) {
						percent = Math.min(100, Math.round((downloaded / total) * 100));
					}
				}
			});
			await relaunch();
		} catch (e) {
			toast.error(installUpdateMessage(e));
			installing = false;
		}
	}
</script>

{#if update && !dismissed}
	<div class="flex flex-wrap items-center gap-2 border-b bg-accent px-3 py-2 text-sm">
		<p class="min-w-0 flex-1">
			Version {update.version} disponible.
			{#if installing && percent !== null}
				<span class="font-mono tabular-nums opacity-80">{percent}%</span>
			{/if}
		</p>
		<Button size="sm" disabled={installing} onclick={() => void install()}>Installer</Button>
		<Button size="sm" variant="ghost" disabled={installing} onclick={() => (dismissed = true)}>
			Plus tard
		</Button>
	</div>
{/if}
```

Si `DownloadEvent` n’a pas `contentLength` sur `Progress` (seulement `Started`), calculer le % uniquement depuis `Started.data.contentLength` stocké dans une variable locale du `install()`, pas dans `onDownloadEvent`. Adapter le switch pour compiler (`bun run check`) : le `default` `never` doit matcher l’union réelle du package.

Correction si le type `Progress` n’a que `chunkLength` : garder `contentLength` depuis l’événement `Started` :

Dans `install()` uniquement (retirer le % de `onDownloadEvent` Progress si besoin) :

```ts
let contentLength = 0;
let downloaded = 0;
await update.downloadAndInstall((event) => {
	switch (event.event) {
		case 'Started':
			contentLength = event.data.contentLength ?? 0;
			downloaded = 0;
			percent = 0;
			break;
		case 'Progress':
			downloaded += event.data.chunkLength;
			if (contentLength > 0) {
				percent = Math.min(100, Math.round((downloaded / contentLength) * 100));
			}
			break;
		case 'Finished':
			percent = 100;
			break;
		default: {
			const _exhaustive: never = event;
			void _exhaustive;
		}
	}
});
```

Dans ce cas, supprimer `onDownloadEvent` du composant pour n’avoir qu’un switch.

- [ ] **Step 3: Layout**

`src/routes/+layout.svelte` : importer `UpdateBanner` et le placer dans `SidebarInset` **sous** le `<header>`, **au-dessus** du `div` scrollable :

```svelte
	import UpdateBanner from '$lib/components/UpdateBanner.svelte';
```

```svelte
		<header class="flex h-10 shrink-0 items-center gap-2 border-b px-2">
			<Sidebar.Trigger />
		</header>
		<UpdateBanner />
		<div class="relative min-h-0 flex-1 overflow-auto">
			{@render children()}
		</div>
```

- [ ] **Step 4: Vérifier**

```bash
bun run check
bun test src/lib/updater-errors.test.ts
```

Expected: 0 erreur TS, tests PASS.

En `tauri dev`, le bandeau ne s’affiche pas (`probeUpdate` → `null`).

- [ ] **Step 5: Commit**

```bash
git add src/lib/updater.ts src/lib/components/UpdateBanner.svelte src/routes/+layout.svelte
git commit -m "$(cat <<'EOF'
feat: bandeau de mise à jour au démarrage

EOF
)"
```

---

### Task 4: Workflow GitHub Release `.deb`

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `tauri.conf.json` version, secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `SYNAPT_UPDATER_TOKEN`
- Produces: workflow `push` tags `v[0-9]+.[0-9]+.[0-9]+`, `tauri-action`, `--bundles deb`, release non-draft (repo privé)

- [ ] **Step 1: Créer `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

jobs:
  release:
    permissions:
      contents: write
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libssl-dev

      - name: Set up Bun
        uses: oven-sh/setup-bun@v2

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Install frontend dependencies
        run: bun install

      - name: Build and release
        uses: tauri-apps/tauri-action@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          SYNAPT_UPDATER_TOKEN: ${{ secrets.SYNAPT_UPDATER_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Synapt v__VERSION__'
          releaseBody: 'Paquet Debian et latest.json pour l’updater.'
          releaseDraft: false
          prerelease: false
          args: --bundles deb
```

Ne pas mettre de PAT ni de clé dans le fichier.

- [ ] **Step 2: Vérifier le YAML**

```bash
python3 -c "import pathlib,yaml; yaml.safe_load(pathlib.Path('.github/workflows/release.yml').read_text()); print('ok')"
```

Si PyYAML absent :

```bash
python3 -c "import pathlib; p=pathlib.Path('.github/workflows/release.yml').read_text(); assert 'args: --bundles deb' in p and 'SYNAPT_UPDATER_TOKEN' in p and 'releaseDraft: false' in p; print('ok')"
```

Expected: `ok`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "$(cat <<'EOF'
ci: publier le .deb signé sur tag v*

EOF
)"
```

Hors git (humain, après merge) : secrets GitHub `TAURI_SIGNING_PRIVATE_KEY` (contenu de `~/.tauri/synapt.key`), `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (vide si `--password ''`), `SYNAPT_UPDATER_TOKEN` (PAT lecture releases). Tag `v0.1.0` aligné sur `tauri.conf.json` / `package.json`.

---

## Self-review

- Spec canal privé + PAT Rust : Task 1 + 4
- Bandeau Installer / Plus tard : Task 3
- CI tag + `.deb` : Task 4
- Check silencieux / toasts install : Task 2 + 3
- Hors scope Réglages / ntfy / AppImage : aucun task
- Permission relance : `process:allow-restart` (identifiant réel Tauri)
