# Synapt — mises à jour automatiques (`.deb`)

Application déjà installée via paquet Debian. L’utilisateur est notifié dans l’app, confirme, puis le `.deb` est installé (Polkit) et l’app relance.

## Décisions

- Canal : GitHub Releases **privées** (`sylecium/Synapt`) + PAT lecture dans le binaire de **release** uniquement
- UX : bandeau Installer / Plus tard (pas de téléchargement ni d’install sans confirmation)
- Publication : GitHub Actions sur tag `v*`
- Format : `.deb` uniquement pour l’updater (pas AppImage, pas Windows/macOS)

## Hors périmètre

- AppImage / RPM / Flatpak
- Check manuel dans Réglages
- Notification ntfy des mises à jour
- Dépôt APT
- Rotation de token dans l’UI
- Mises à jour silencieuses

## Architecture

Plugins officiels Tauri 2 : `tauri-plugin-updater` (≥ 2.10, install `.deb`) et `tauri-plugin-process` (relaunch).

Le PAT n’est jamais dans le frontend ni dans git. En release, Rust lit `SYNAPT_UPDATER_TOKEN` (`option_env!`) et pose le header `Authorization: Bearer …` sur le plugin. En `tauri dev` ou sans variable : aucun check.

Endpoint : `https://github.com/sylecium/Synapt/releases/latest/download/latest.json`.

La pubkey minisign (contenu base64, pas un chemin) est dans `tauri.conf.json` (`plugins.updater.pubkey`). La clé privée et son mot de passe sont des secrets GitHub (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). Génération locale une fois : `bunx tauri signer generate`.

`bundle.createUpdaterArtifacts: true`. L’app lancée depuis un `.deb` cible la plateforme `linux-x86_64-deb`. SQLite et réglages (XDG) ne sont pas touchés par `dpkg -i` du même paquet.

## Composants

| Pièce | Rôle |
|---|---|
| `src-tauri` (plugin updater) | Header auth, vérif signature, download, `pkexec dpkg -i` |
| `src-tauri` (plugin process) | Relance après install |
| `src/lib/updater.ts` | `check()` au démarrage prod |
| `UpdateBanner.svelte` | Bandeau layout : version, Installer, Plus tard, progression |
| `+layout.svelte` | Monte le bandeau |
| `.github/workflows/release.yml` | Tag `v*` → build `.deb` signé + `latest.json` + release privée |
| capabilities | `updater:default`, `process:allow-relaunch` |

CI : Ubuntu, Bun, `tauri-action`, `--bundles deb`. Secrets : signing + `SYNAPT_UPDATER_TOKEN` (même PAT que celui cuit dans le binaire, lecture Contents/releases du repo). `GITHUB_TOKEN` suffit pour créer la release.

Le PAT est un jeton fine-grained, lecture seule des releases du repo Synapt. Quiconque a le `.deb` peut l’extraire : accepté pour un usage solo.

## Flux

1. Démarrage production → `check()` (silence si pas de token, déjà à jour, ou échec réseau/auth au check).
2. Mise à jour disponible → bandeau.
3. Plus tard → bandeau masqué jusqu’au prochain lancement.
4. Installer → download + vérif signature → Polkit (`pkexec dpkg -i`) → `relaunch()`.

## Erreurs

Check au démarrage : échec silencieux. Toasts uniquement après clic Installer, en français :

- Réseau ou GitHub : « Impossible de télécharger la mise à jour. »
- Auth GitHub : « Accès aux mises à jour refusé. »
- Signature invalide : « La mise à jour n’est pas fiable, installation annulée. »
- Polkit annulé ou `dpkg` échoué : « Installation de la mise à jour annulée. »

Pas de jargon IPC, HTTP, PAT, signature minisign.

## Tests

- Sans `SYNAPT_UPDATER_TOKEN` (dev) : aucun appel réseau updater.
- Pas de test d’install `.deb` en CI (Polkit + release réelle).
- Vérif manuelle après première release taguée : bandeau, Plus tard, Installer + mot de passe, relance, version affichée.

## Setup une fois (humain)

1. `bunx tauri signer generate -w ~/.tauri/synapt.key`
2. Coller la pubkey dans `tauri.conf.json`
3. Secrets GitHub : clé privée, mot de passe, PAT updater
4. Premier tag `vX.Y.Z` aligné sur `tauri.conf.json` / `package.json`
