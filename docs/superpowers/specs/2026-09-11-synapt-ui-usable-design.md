# Synapt UI utilisable — spec

Rendre le MVP quotidiennement utilisable : parcours (lot A) + identité visuelle desktop (lot B). Pas de nouvelle feature métier.

## Hors périmètre

- Lot C (file du jour en gros, note post-séance)
- Facture PDF, cloud, email/SMS, notifs bureau
- GSAP, Framer, glassmorphism, bento marketing
- Superforms, nouvelles libs hors shadcn-svelte + fontsource
- Refonte backend (sauf câblage UI de `tarifs_set_actif(true)`, déjà exposé)

## Identité (lot B)

Outil desktop Linux, 8 h/jour, pas un site marketing. Densité haute, motion minimale (`hover`/`active` seulement, `prefers-reduced-motion` respecté).

**Signature :** grille agenda type tableau horaire. Heures en mono, colonne « aujourd’hui » teintée, RDV en blocs pleins. Un seul accent.

### Tokens

| Rôle | Valeur |
|---|---|
| Paper | `#F3F1EC` (oklch ~ 0.96 0.008 95) |
| Ink | `#1C1915` (oklch ~ 0.22 0.015 70) |
| Line | `#E4E0D8` |
| Pine (accent) | `#2F6A5A` (oklch ~ 0.48 0.07 165) |
| Pine ink | `#F4F7F5` |
| Warn (hors plage) | `#8A5A2B` fond `#F4E6D4` |
| Radius | `0.4rem` |
| Page padding | `p-4` (plus de `p-6` systématique) |

Dark : paper `#1A1916`, ink `#EDEAE4`, pine `#6FA894`, sidebar un cran plus sombre que le fond.

`--primary` = pine. Plus de primary noir mira. Plus d’Inter.

### Type

- UI : `@fontsource-variable/geist` (`font-sans`)
- Heures, durées, prix, dates : `@fontsource-variable/geist-mono` (`font-mono tabular-nums`)
- Titres page : `text-lg font-semibold tracking-tight` (pas `text-2xl`)
- Labels section : `text-xs font-medium uppercase tracking-wide text-muted-foreground`

### Chrome

```
[ sidebar icônes|libellés ] [ trigger | titre page          actions ]
                             [ contenu dense, listes divide-y        ]
```

- `SidebarProvider` + `collapsible="icon"` + `SidebarTrigger` dans un header sticky de l’inset
- Toggle clair/sombre (mode-watcher) dans le footer sidebar
- Pas de cartes empilées pour les listes : `divide-y` + hover row
- Cartes seulement pour empty states et bandeaux

## Parcours (lot A)

### Premier lancement

Sur `/` uniquement, si 0 tarif **ou** 0 client : panneau `EmptyState` avec 3 étapes numérotées, chacune un bouton.

1. Créer un tarif → `/tarifs`
2. Créer un client → `/clients`
3. Créer un RDV → ouvre `RdvDialog` (désactivé tant que 1 ou 2 manque)

Le bandeau ntfy/Stripe reste, en dessous, non bloquant.

### États partagés

Composant `EmptyState.svelte` : titre, une phrase d’action, CTA optionnel.

Composant `PageHeader.svelte` : `SidebarTrigger` n’est pas ici (il vit dans le layout). Header de page = titre + actions à droite.

Skeletons (`Skeleton` shadcn) pendant le premier fetch de chaque page. Pas de spinner centré.

### Dashboard `/`

- Clic sur la ligne RDV (aujourd’hui ou à venir) ouvre `RdvPanel` (même sheet que l’agenda)
- Jitsi : un bouton primaire sur la ligne du jour seulement. Copier / Stripe vivent dans le panneau, plus sur la ligne
- Empty aujourd’hui : « Aucun RDV aujourd’hui » + bouton Nouveau RDV
- Empty à venir : une ligne muted, pas de faux tableau

### Agenda `/agenda`

Barre :

```
[Aujourd’hui]  [<]  8–14 sept.  [>]   [Semaine | Jour]   [date popover]   [Nouveau RDV]
```

- Préc/suiv avance d’une semaine (vue semaine) ou d’un jour (vue jour)
- Le calendrier Bits n’occupe plus une colonne permanente : `Popover` au clic sur la date
- Colonne aujourd’hui : fond pine/8, header souligné pine
- Blocs RDV : fond pine, texte pine-ink, nom + heure mono
- Hors plage : bandeau warn sous la grille, cliquable, heure visible

### RDV dialog

- Client : combobox (Popover + Command shadcn, recherche nom). Si 0 client, lien vers `/clients`
- Tarif : `Select` shadcn (actifs seulement). Changement de tarif met à jour la durée
- Début : `datetime-local` conservé, classe Input existante
- Checkboxes natives remplacées nulle part ici ; dans Réglages : `Switch` shadcn

### Clients / tarifs

- Ligne entière cliquable (clients → fiche ; tarifs → dialog édition)
- Empty clients / tarifs : `EmptyState` + CTA créer
- Tarif inactif : bouton « Réactiver » (`tarifsSetActif(id, true)` déjà dans `api.ts`)

### Notes

- Empty : `EmptyState` « Première note personnelle »
- Liste : titre = 1re ligne du corps (tronquée), date muted, clic pour éditer in place. Plus une pile de textareas toujours ouvertes

### Réglages

Aide sous les champs, français concret :

- Topic : « Nom secret du canal. Sans topic, les rappels téléphone sont coupés. »
- Token : « Optionnel, si le serveur ntfy l’exige. »
- Clé Stripe : « Collez une clé secrète `sk_…`. Elle reste sur cette machine. »

`Switch` pour rappels 24 h / 1 h.

### Fiche client

Ligne historique cliquable → `RdvPanel` (pas seulement du texte). Empty notes / historique avec phrase d’action.

## Composants à ajouter (shadcn-svelte)

`select`, `popover`, `command`, `switch`, `checkbox` (si Switch ne couvre pas). `combobox` = composition Popover + Command, pas un primitive séparé.

Nouveaux fichiers app :

- `src/lib/components/EmptyState.svelte`
- `src/lib/components/PageHeader.svelte`
- `src/lib/components/ClientCombobox.svelte`

## Contraintes

- Frontend Svelte 5, invoke Rust inchangé (sauf UI réactivation tarif)
- Bun pour install fonts / CLI shadcn
- Copy sentence case, verbes concrets, pas d’emoji
- Vérif : `bun run check` + parcours manuel dashboard → RDV → panneau → agenda
