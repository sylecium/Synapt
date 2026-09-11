# Synapt MVP — spec

Application desktop Linux (Debian 13) pour un usage solo : dashboard, agenda, fiches clients, tarifs, notes, liens Jitsi, Payment Links Stripe, rappels ntfy sur téléphone.

## Hors périmètre

- Facture PDF, Factur-X, réforme 2026
- Cloud, sync, multi-utilisateurs, multi-devises
- Email, SMS, notifications bureau Linux
- Hub d’autres applications
- Superforms / formsnap, Radix React, tauri-plugin-sql, tauri-plugin-notification
- Schedule-X, SVAR, FullCalendar

## Stack

- Tauri 2, cible Linux
- SvelteKit 2 + Svelte 5, SPA : `@sveltejs/adapter-static`, `ssr = false`, `prerender = false`, fallback `index.html`
- Tailwind 4 (`@tailwindcss/vite`), shadcn-svelte style `new-york`, Bits UI
- `@lucide/svelte`, `svelte-sonner`, `mode-watcher`, `@internationalized/date`
- Bun pour install et scripts frontend
- Rust : rusqlite, reqwest (Stripe + ntfy)
- Plugins Tauri : `opener`, `clipboard-manager`
- État UI : runes Svelte 5 (`$state`, modules `.svelte.ts`), pas de lib de store
- Formulaires : `$state` + Zod côté client, `invoke` Rust (pas d’actions SvelteKit serveur)

Scaffold : `bunx create-tauri-app` puis Kit en SPA + shadcn-svelte. Commandes : `bun`.

## Architecture

Un process desktop. Le frontend n’exécute pas de SQL et n’appelle ni Stripe ni ntfy. Toute I/O sensible passe par `invoke`.

- SQLite : `~/.local/share/synapt/synapt.db` (XDG data)
- Réglages secrets : `~/.config/synapt/settings.json`, mode `0600`
- Datetimes stockés en UTC (ISO 8601), affichés en fuseau local
- IDs : UUID v4 en texte
- Prix : entier en centimes EUR, jamais de float

## Écrans

Sidebar shadcn `new-york` :

| Route | Rôle |
|---|---|
| `/` | Dashboard (accueil) |
| `/agenda` | Semaine / jour |
| `/clients` | Liste + fiche |
| `/tarifs` | Grille tarifaire |
| `/notes` | Notes personnelles |
| `/reglages` | ntfy + Stripe |

### Dashboard `/`

- Liste des RDV du jour (heure, client, tarif) : ouvrir Jitsi, copier/ouvrir Stripe, ouvrir la fiche client
- Les 5 prochains RDV hors aujourd’hui
- Actions : Nouveau RDV (même dialog que l’agenda), lien vers l’agenda
- Bandeau si ntfy (topic) ou Stripe (clé) manquant, lien vers `/reglages`
- Pas de CA, pas de stats, pas d’impayés, pas de notes récentes

### Agenda `/agenda`

- Vue semaine par défaut, bascule jour / semaine
- Grille maison (Tailwind + Calendar Bits pour le date picker), pas de lib scheduler
- Clic créneau vide : dialog Nouveau RDV (client, tarif, début, durée reprise du tarif, note courte)
- Clic RDV : panneau latéral (pas une route dédiée) : client, horaire, Jitsi (copier / ouvrir), Stripe (créer si absent, copier / ouvrir), note, état rappels ntfy, modifier, annuler
- RDV annulés invisibles dans la grille

### Clients `/clients`

Liste + recherche. Fiche : nom, email, téléphone, notes client, historique RDV, bouton Nouveau RDV.

### Tarifs `/tarifs`

Tableau : nom, durée minutes, prix TTC. CRUD dialog. Tarif inactif : plus proposé à la création de RDV, les RDV existants le conservent.

### Notes `/notes`

Notes perso (`client_id` null). Les notes client vivent uniquement sur la fiche.

### Réglages `/reglages`

- ntfy : serveur (défaut `https://ntfy.sh`), topic, token optionnel, cases rappel 24 h et 1 h (les deux cochées par défaut), bouton Tester
- Stripe : clé secrète, champ masqué
- Sans topic ntfy : rappels désactivés, app OK
- Sans clé Stripe : boutons paiement inactifs, app OK

## Modèle SQLite

### clients

- `id` TEXT PK
- `nom` TEXT NOT NULL
- `email` TEXT
- `telephone` TEXT
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### tarifs

- `id` TEXT PK
- `nom` TEXT NOT NULL
- `duree_minutes` INTEGER NOT NULL CHECK (`duree_minutes` > 0)
- `prix_centimes` INTEGER NOT NULL CHECK (`prix_centimes` >= 0)
- `actif` INTEGER NOT NULL DEFAULT 1
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### rdv

- `id` TEXT PK
- `client_id` TEXT NOT NULL REFERENCES clients(id)
- `tarif_id` TEXT REFERENCES tarifs(id)
- `debut` TEXT NOT NULL
- `duree_minutes` INTEGER NOT NULL CHECK (`duree_minutes` > 0)
- `jitsi_url` TEXT NOT NULL
- `stripe_url` TEXT
- `stripe_id` TEXT
- `note` TEXT
- `statut` TEXT NOT NULL CHECK (`statut` IN ('planifie', 'annule'))
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### notes

- `id` TEXT PK
- `client_id` TEXT REFERENCES clients(id)
- `corps` TEXT NOT NULL
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

`client_id` null = note perso.

### rappels_ntfy

- `id` TEXT PK
- `rdv_id` TEXT NOT NULL REFERENCES rdv(id)
- `type` TEXT NOT NULL CHECK (`type` IN ('24h', '1h'))
- `ntfy_id` TEXT
- `echeance` TEXT NOT NULL
- `etat` TEXT NOT NULL CHECK (`etat` IN ('programme', 'annule'))
- UNIQUE (`rdv_id`, `type`)

Suppression client : interdite s’il reste des RDV `planifie`. Notes client et RDV `annule` : le MVP refuse la suppression client tant qu’il existe une fiche liée (historique). Alternative acceptée : pas de bouton supprimer client dans le MVP (seulement édition). **Décision : pas de suppression client dans le MVP**, édition seulement.

Suppression tarif : désactivation (`actif = 0`), pas de DELETE.

## Réglages fichier

`~/.config/synapt/settings.json` :

```json
{
  "ntfy": {
    "serveur": "https://ntfy.sh",
    "topic": "",
    "token": "",
    "rappel_24h": true,
    "rappel_1h": true
  },
  "stripe": {
    "secret_key": ""
  }
}
```

Jamais renvoyé au frontend en clair dans les listes. L’UI de réglages charge via une commande `settings_get` qui masque la clé Stripe (dernier 4 caractères ou booléen `configured`). L’écriture passe par `settings_set`. Token ntfy masqué de la même façon.

## Règles métier

### Jitsi

À la création du RDV : `https://meet.jit.si/synapt-<uuid-du-rdv>`. Immutable ensuite.

### Conflits

Deux RDV `planifie` dont les intervalles `[debut, debut + duree)` se chevauchent : refus. Les `annule` ne comptent pas.

### Stripe

Payment Link via API Stripe, montant = `prix_centimes` du tarif du RDV, devise EUR. Si pas de tarif ou prix 0 : pas de lien.

Si la clé Stripe est présente à `rdv_create` : tenter la création du lien tout de suite. Sinon (clé absente ou échec) : `stripe_url` vide ; le bouton du panneau appelle `stripe_ensure_link`. Échec Stripe : RDV quand même créé, toast.

Ouvrir une URL : plugin `opener` depuis le frontend. Copier : plugin `clipboard-manager` depuis le frontend.

### ntfy

`POST {serveur}/{topic}` (JSON ou headers `Title`, `Message`, `Delay`/`At`, `Click` = url Jitsi, `Priority` haute pour `1h`). `Authorization: Bearer` si token.

Programmer un rappel seulement si :

- topic non vide
- case correspondante cochée
- RDV `planifie`
- échéance dans le futur
- délai jusqu’à l’échéance ≤ 3 jours (limite ntfy.sh)
- pas déjà `programme` pour ce `(rdv_id, type)`

Échéances : `debut - 24h` et `debut - 1h`.

Si l’échéance est à plus de 3 jours : ne rien publier maintenant. Au démarrage de l’app et toutes les heures, scanner les RDV `planifie` et publier ceux qui entrent dans la fenêtre.

Annulation RDV : `DELETE {serveur}/{topic}/{ntfy_id}` pour chaque rappel `programme`, puis `etat = annule`. Échec DELETE : on marque quand même `annule` en local, toast.

Bouton Tester : message immédiat « Synapt OK ».

Échec ntfy à la création du RDV : RDV créé, toast, pas de rollback.

## Commandes Tauri (contrat)

Noms stables, payloads JSON snake_case.

- `settings_get` / `settings_set`
- `clients_list` / `clients_get` / `clients_upsert`
- `tarifs_list` / `tarifs_upsert` / `tarifs_set_actif`
- `notes_list` (filtre optionnel `client_id` ; `perso: true` pour les notes sans client) / `notes_upsert` / `notes_delete`
- `rdv_list` (plage `from`/`to` UTC) / `rdv_get` / `rdv_create` / `rdv_update` / `rdv_annuler`
- `rdv_dashboard` : `{ aujourdhui: Rdv[], a_venir: Rdv[] }` (5 max pour à venir)
- `stripe_ensure_link(rdv_id)` : crée le Payment Link si absent
- `ntfy_test`

`ntfy_sync` n’est pas exposé au frontend : au `setup` Tauri, puis timer interne 1 h.

## Erreurs UI

- Toasts sonner pour les échecs réseau Stripe/ntfy
- Bandeau dashboard si config incomplète
- Conflit horaire : message dans le dialog, pas de création
- Réglages illisibles : défauts vides, app utilisable

## Tests

Un binaire/test Rust, SQLite en mémoire (ou fichier temp) :

1. Créer client + tarif + RDV → `jitsi_url` non vide, statut `planifie`
2. Second RDV chevauchant → erreur
3. `rdv_annuler` → statut `annule` ; un chevauchement sur le même créneau est alors accepté
4. Pas d’appel réseau réel Stripe/ntfy (fonctions de planification pures : échéance 24 h / 1 h, fenêtre 3 jours)

Lancer : `cargo test` dans `src-tauri`.

## Critère « MVP utilisable »

Sur Debian 13 : `bun run tauri dev` ouvre l’app. On crée un client, un tarif, un RDV depuis le dashboard ou l’agenda, on copie/ouvre Jitsi, on enregistre une note perso et une note client. Stripe et ntfy marchent une fois configurés, et n’empêchent pas le reste si vides.
