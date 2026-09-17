# Synapt — notes d’honoraires PDF

Note d’honoraires B2C, une séance ou plusieurs, figée à l’émission. Inspirée du flux Invoicing Odoo (créer, numéroter, PDF, acquitter), pas du logiciel. Pas de compta, pas de Factur-X.

## Décisions

- Une note = 1 séance (défaut) ou N séances du **même** client
- Document figé : numéro, snapshots, PDF sur disque
- Toujours **acquittée** à la génération ; le moyen de paiement est choisi (espèces, chèque, CB, Stripe)
- Annulation : le numéro reste, statut `annulee`, bandeau ANNULÉE, les RDV redeviennent disponibles
- Un RDV n’est sur **aucune** note `emise` à la fois
- SQLite = vérité ; `~/Synapt/honoraires/` = fichiers PDF
- UI : `/honoraires` + fiche client + panneau / clic droit RDV
- SIRET optionnel, non bloquant. Génération bloquée seulement si le **nom** du cabinet est vide

## Hors périmètre

- Facture électronique, Factur-X, réforme 2026
- Avoir, grand livre, TVA calculée, rapprochement bancaire
- Email / SMS d’envoi
- Logo, ADELI / RPPS
- Retour de paiement Stripe (webhook)
- Intégration ou copie du code Odoo
- Impression sans fichier, aperçu WYSIWYG

## Architecture

Même contrat que le reste de l’app : frontend Svelte via `invoke`, pas de SQL côté UI. PDF généré en Rust (crate A4, police Unicode embarquée, accents français). Ouverture via `@tauri-apps/plugin-opener` (`openPath`).

- DB : `~/.local/share/synapt/synapt.db`
- Réglages cabinet : `~/.config/synapt/settings.json` (clé `cabinet`, défauts si absente)
- PDF : `~/Synapt/honoraires/{numero}.pdf` (`create_dir_all` au premier enregistrement)

Si le fichier manque à l’ouverture : régénération depuis le snapshot, même contenu, puis ouverture. Pas de toast dans ce cas.

## Réglages `cabinet`

```json
{
  "cabinet": {
    "nom": "",
    "adresse": "",
    "telephone": "",
    "email": "",
    "siret": "",
    "mention_tva": "TVA non applicable, art. 261-4-1° du CGI",
    "prefixe_numero": ""
  }
}
```

`settings_get` / `settings_set` exposent ces champs (pas secret). `prefixe_numero` : vide ou `[A-Za-z0-9_-]{1,12}`. S’il est non vide et ne finit pas par `-`, on insère un `-` à la concaténation (`NH` → `NH-2026-0001`). Vide → `2026-0001`.

## Modèle SQLite

### `clients`

Colonne ajoutée : `adresse` TEXT (optionnel), même migration `ALTER` que les champs dossier.

### `honoraires`

- `id` TEXT PK
- `numero` TEXT NOT NULL UNIQUE
- `client_id` TEXT NOT NULL REFERENCES clients(id)
- `client_nom` TEXT NOT NULL
- `client_date_naissance` TEXT
- `client_adresse` TEXT
- `cabinet_nom` TEXT NOT NULL
- `cabinet_adresse` TEXT
- `cabinet_telephone` TEXT
- `cabinet_email` TEXT
- `cabinet_siret` TEXT
- `mention_tva` TEXT NOT NULL
- `moyen_paiement` TEXT NOT NULL CHECK (`moyen_paiement` IN ('especes', 'cheque', 'cb', 'stripe'))
- `statut` TEXT NOT NULL CHECK (`statut` IN ('emise', 'annulee'))
- `total_centimes` INTEGER NOT NULL CHECK (`total_centimes` >= 0)
- `annee` INTEGER NOT NULL
- `seq` INTEGER NOT NULL
- `pdf_relatif` TEXT NOT NULL
- `created_at` TEXT NOT NULL
- UNIQUE (`annee`, `seq`)

`annee` = année civile du fuseau local à l’émission. `seq` monotonic pour l’année, **y compris** après annulation (jamais réutilisé). Compteur : `MAX(seq) WHERE annee = ?` + 1, dans la même transaction que l’insert.

`numero` = `{prefixe}{annee}-{seq:04}` (ex. `2026-0001`, `NH-2026-0001`). `pdf_relatif` = `honoraires/{numero}.pdf`.

### `honoraire_lignes`

- `id` TEXT PK
- `honoraire_id` TEXT NOT NULL REFERENCES honoraires(id)
- `rdv_id` TEXT NOT NULL REFERENCES rdv(id)
- `debut` TEXT NOT NULL
- `duree_minutes` INTEGER NOT NULL
- `tarif_nom` TEXT NOT NULL
- `prix_centimes` INTEGER NOT NULL
- `actif` INTEGER NOT NULL DEFAULT 1
- UNIQUE INDEX partiel : `honoraire_lignes(rdv_id) WHERE actif = 1`

À l’annulation de la note : `statut = annulee` et `actif = 0` sur toutes les lignes.

### Suppression client

Refus s’il existe au moins une note (`emise` ou `annulee`) pour ce client. Message : « Ce client a encore des notes d’honoraires. »

Les snapshots restent exploitables tant que la fiche existe. Pas de cascade sur `honoraires`.

## Règles métier

`honoraires_create(rdv_ids, moyen_paiement)` :

1. `rdv_ids` non vide, sans doublon
2. `cabinet.nom` trim non vide, sinon erreur
3. Tous les RDV existent, `statut = planifie`, même `client_id`
4. Aucun n’a déjà une ligne `actif = 1`
5. Tarif absent : `tarif_nom` = « Séance », `prix_centimes` = 0 (les RDV futurs sans tarif sont autorisés)
6. Snapshot client (nom, naissance, adresse) et cabinet au moment T
7. Lignes triées par `debut` croissant
8. `total_centimes` = somme des lignes
9. Insert en transaction, écriture PDF, `commit`. Échec PDF ou IO : `rollback`, pas de ligne en base

Toujours acquittée. Pas d’état « en attente ».

`honoraires_annuler` : si déjà `annulee`, message « Cette note est déjà annulée. » Sinon statut + lignes `actif = 0` + réécriture PDF avec bandeau ANNULÉE.

## PDF

A4 portrait, noir sur blanc, police avec accents. Une page si ça tient, pages suivantes si trop de lignes. Pas de tiret cadratin.

1. Titre **Note d’honoraires**, numéro, date d’émission (locale)
2. Colonne cabinet (nom, adresse, tél, email, SIRET si rempli)
3. Colonne client (nom, naissance et adresse si remplis)
4. Tableau : date, horaire, intitulé, durée, montant
5. Total
6. **Acquittée** + libellé du moyen (Espèces, Chèque, Carte, Stripe)
7. Mention TVA
8. Bloc **Signature**

Champs vides omis (pas de « - »). Note `annulee` : bandeau diagonal **ANNULÉE**, contenu inchangé.

## Écrans

Sidebar : **Honoraires** après Clients, route `/honoraires`.

### `/honoraires`

Liste (plus récente en tête) : numéro, date, client, total, statut (`Émise` / `Annulée`). Clic = ouvrir le PDF. Clic droit : Ouvrir ; Annuler si `emise` (dialog de confirmation). Bouton **Nouvelle note** : dialog client (combobox existant), séances `planifie` du client non déjà sur une note émise (futures incluses), moyen de paiement.

### Fiche client

- Adresse : champ optionnel via « Ajouter des champs », comme les autres extras
- Bloc notes d’honoraires (liste, ouvrir, annuler)
- **Regrouper** : même dialog, client déjà fixé

### RDV (panneau + clic droit)

- Pas de ligne `actif` : **Note d’honoraires** → dialog moyen (1 séance)
- Note `emise` : **Ouvrir la note**
- Note uniquement `annulee` : on peut en recréer une

### Réglages

Bloc **Cabinet** en tête : nom, adresse, tél, email, SIRET, mention TVA, préfixe. Nom vide : la génération toaste et propose `/reglages`.

## Commandes Tauri

Noms `snake_case`.

- `honoraires_list(client_id?: string)`
- `honoraires_get(id)` : note + lignes
- `honoraires_create({ rdv_ids: string[], moyen_paiement })` → Honoraire
- `honoraires_ouvrir(id)` : régénère si fichier absent, retourne le chemin absolu
- `honoraires_annuler(id)` → Honoraire
- `honoraires_rdvs_disponibles(client_id)` : RDV `planifie` sans ligne `actif`

`settings_get` / `settings_set` / `clients_upsert` étendus (cabinet, `adresse`).

Le frontend ouvre le chemin renvoyé par `honoraires_ouvrir`. Il ne lit pas `~/Synapt/` lui-même.

## Erreurs UI

Français métier, toasts / dialog, pas de jargon SQL/IPC.

| Cas | Message |
|---|---|
| Nom cabinet vide | Indiquez le nom du cabinet dans les réglages. |
| RDV déjà sur une note émise | Cette séance a déjà une note d’honoraires. |
| Clients différents | Une note ne peut concerner qu’un seul client. |
| RDV annulé | Impossible d’inclure un rendez-vous annulé. |
| Liste vide | Choisissez au moins une séance. |
| Note déjà annulée | Cette note est déjà annulée. |
| Dossier `~/Synapt/` | Impossible d’enregistrer le PDF dans le dossier Synapt. |
| Ouverture PDF après échec regen | Le PDF n’a pas pu être ouvert. |
| Client avec honoraires | Ce client a encore des notes d’honoraires. |

PDF manquant puis régénéré avec succès : pas de message.

## Tests

`cargo test` dans `src-tauri`, SQLite mémoire, dossier PDF temp (pas `~/Synapt` réel dans les tests).

1. 1 séance → `2026-0001` (ou année du test), snapshot, fichier
2. 2e note → `seq + 1` ; préfixe vide et préfixe `NH`
3. Annuler → `annulee`, lignes `actif = 0`, RDV libéré, 3e note **n’est pas** le numéro annulé
4. Refus : RDV déjà émis, clients mélangés, cabinet sans nom, `rdv_ids` vide
5. 2 RDV même client → total = somme, 2 lignes
6. Ouvrir après suppression du fichier → régénère le même `numero`
7. `clients_delete` refusé s’il reste une note
8. RDV sans tarif accepté, `prix_centimes = 0`

Pas de test Playwright. Pas de test pixel du PDF.

## Critère de fin

Sur Debian 13 : réglages cabinet remplis, on émet une note depuis un RDV, on l’ouvre depuis `/honoraires`, on en émet une groupée sur la fiche client, on annule, on réémet, les numéros ne se réutilisent pas, le PDF groupé et le PDF ANNULÉE sont dans `~/Synapt/honoraires/`.
