# Synapt UI utilisable Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre le MVP quotidiennement utilisable : parcours (lot A) + identité visuelle desktop (lot B), sans nouvelle feature métier.

**Architecture:** Tokens et typo dans `src/app.css`. Chrome (sidebar collapse, trigger, thème) dans le layout. Composants partagés `EmptyState`, `PageHeader`, `ClientCombobox`. Chaque écran consomme ces pièces. Backend inchangé ; `tarifsSetActif(id, true)` est déjà exposé.

**Tech Stack:** Tauri 2, SvelteKit SPA, Svelte 5, Tailwind 4, shadcn-svelte (mira), Bits UI, Bun, `@fontsource-variable/geist` + `geist-mono`.

**Spec:** `docs/superpowers/specs/2026-09-11-synapt-ui-usable-design.md`

## Global Constraints

- Frontend Svelte 5, invoke Rust inchangé (sauf UI réactivation tarif via `tarifsSetActif` déjà dans `src/lib/api.ts`)
- Bun uniquement pour install / scripts JS (`bun add`, `bun x shadcn-svelte@latest add …`, `bun run check`)
- Nouvelles libs autorisées : `@fontsource-variable/geist`, `@fontsource-variable/geist-mono`, composants shadcn `select` `popover` `command` `switch`
- Pas de GSAP, Framer, Superforms, Inter, primary noir mira
- `--primary` = pine `#2F6A5A` (dark `#6FA894`). Paper `#F3F1EC`, ink `#1C1915`, line `#E4E0D8`, pine ink `#F4F7F5`, warn `#8A5A2B` / fond `#F4E6D4`. Radius `0.4rem`. Page padding `p-4`
- Dark : paper `#1A1916`, ink `#EDEAE4`, pine `#6FA894`, sidebar un cran plus sombre
- UI Geist (`font-sans`), heures/durées/prix/dates Geist Mono (`font-mono tabular-nums`)
- Titres page `text-lg font-semibold tracking-tight`. Labels section `text-xs font-medium uppercase tracking-wide text-muted-foreground`
- Copy sentence case, verbes concrets, pas d’emoji
- Motion minimale : hover/active seulement, `prefers-reduced-motion` respecté
- Listes : `divide-y` + hover row, pas de cartes empilées. Cartes seulement empty states et bandeaux
- Vérif par tâche : `bun run check` (0 erreur). Pas de nouveau test runner
- Ne pas modifier `docs/superpowers/plans/2026-09-11-synapt-mvp.md` ni le backend Rust
- Commits en français, conventional (`feat:` / `fix:`), un commit par tâche

## File map

- Create: `src/lib/components/EmptyState.svelte`, `PageHeader.svelte`, `ClientCombobox.svelte`
- Modify: `src/app.css`, `src/routes/+layout.svelte`, `src/lib/components/AppSidebar.svelte`, `src/lib/format.ts`
- Modify screens: `src/routes/+page.svelte`, `agenda/+page.svelte`, `clients/+page.svelte`, `clients/[id]/+page.svelte`, `tarifs/+page.svelte`, `notes/+page.svelte`, `reglages/+page.svelte`
- Modify: `src/lib/components/RdvDialog.svelte`, `RdvPanel.svelte`, `WeekGrid.svelte`
- Add via CLI: `src/lib/components/ui/{select,popover,command,switch}/`

---

### Task 1: Tokens, fontes, chrome sidebar

**Files:**
- Modify: `src/app.css`, `src/routes/+layout.svelte`, `src/lib/components/AppSidebar.svelte`
- Create: (none besides font packages)
- Test: `bun run check`

**Interfaces:**
- Consumes: `Sidebar.Root` accepte `collapsible="icon"`. `toggleMode` depuis `mode-watcher`. `Sidebar.Trigger` exporté.
- Produces: thème pine, Geist, layout avec header sticky (trigger) + sidebar collapsible + toggle thème footer.

- [ ] **Step 1: Installer les fontes**

```bash
bun add @fontsource-variable/geist @fontsource-variable/geist-mono
bun remove @fontsource-variable/inter
```

- [ ] **Step 2: Remplacer `src/app.css` tokens**

Remplacer `@import "@fontsource-variable/inter"` par geist + geist-mono.

`:root` (valeurs exactes) :

```css
:root {
	--background: #F3F1EC;
	--foreground: #1C1915;
	--card: #F3F1EC;
	--card-foreground: #1C1915;
	--popover: #F7F5F0;
	--popover-foreground: #1C1915;
	--primary: #2F6A5A;
	--primary-foreground: #F4F7F5;
	--secondary: #E8E4DC;
	--secondary-foreground: #1C1915;
	--muted: #E8E4DC;
	--muted-foreground: #6B6560;
	--accent: #E4EDE9;
	--accent-foreground: #1C1915;
	--destructive: #B42318;
	--border: #E4E0D8;
	--input: #E4E0D8;
	--ring: #2F6A5A;
	--radius: 0.4rem;
	--sidebar: #EDEAE3;
	--sidebar-foreground: #1C1915;
	--sidebar-primary: #2F6A5A;
	--sidebar-primary-foreground: #F4F7F5;
	--sidebar-accent: #E4EDE9;
	--sidebar-accent-foreground: #1C1915;
	--sidebar-border: #E4E0D8;
	--sidebar-ring: #2F6A5A;
	--warn: #8A5A2B;
	--warn-bg: #F4E6D4;
}
```

`.dark` :

```css
.dark {
	--background: #1A1916;
	--foreground: #EDEAE4;
	--card: #1A1916;
	--card-foreground: #EDEAE4;
	--popover: #22211D;
	--popover-foreground: #EDEAE4;
	--primary: #6FA894;
	--primary-foreground: #13241E;
	--secondary: #2A2824;
	--secondary-foreground: #EDEAE4;
	--muted: #2A2824;
	--muted-foreground: #A39E96;
	--accent: #24332E;
	--accent-foreground: #EDEAE4;
	--destructive: #F97066;
	--border: #2F2D28;
	--input: #2F2D28;
	--ring: #6FA894;
	--sidebar: #141310;
	--sidebar-foreground: #EDEAE4;
	--sidebar-primary: #6FA894;
	--sidebar-primary-foreground: #13241E;
	--sidebar-accent: #24332E;
	--sidebar-accent-foreground: #EDEAE4;
	--sidebar-border: #2F2D28;
	--sidebar-ring: #6FA894;
	--warn: #E0B07A;
	--warn-bg: #3A2E22;
}
```

Dans `@theme inline` :

```css
--font-sans: 'Geist Variable', sans-serif;
--font-mono: 'Geist Mono Variable', monospace;
--color-warn: var(--warn);
--color-warn-bg: var(--warn-bg);
```

`@layer base` : garder border/body/html. Ajouter :

```css
@media (prefers-reduced-motion: reduce) {
	*, *::before, *::after {
		animation-duration: 0.01ms !important;
		transition-duration: 0.01ms !important;
	}
}
```

- [ ] **Step 3: Layout chrome**

`src/routes/+layout.svelte` : header sticky dans l’inset avec `Sidebar.Trigger`. Contenu `p-0` (les pages gèrent `p-4` à partir de la tâche 2).

```svelte
<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/AppSidebar.svelte';

	let { children } = $props();
</script>

<ModeWatcher />
<Sidebar.SidebarProvider>
	<AppSidebar />
	<Sidebar.SidebarInset>
		<header class="flex h-10 shrink-0 items-center gap-2 border-b px-2">
			<Sidebar.Trigger />
		</header>
		<div class="min-h-0 flex-1 overflow-auto">
			{@render children()}
		</div>
	</Sidebar.SidebarInset>
	<Toaster />
</Sidebar.SidebarProvider>
```

- [ ] **Step 4: Sidebar collapsible + thème**

`AppSidebar.svelte` : `<Sidebar.Root collapsible="icon">`. Footer :

```svelte
<script lang="ts">
	import { toggleMode } from 'mode-watcher';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	// ...existing nav imports
</script>

<Sidebar.Root collapsible="icon">
	<!-- header + nav inchangés -->
	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={toggleMode} tooltip="Clair / sombre">
					<SunIcon class="dark:hidden" />
					<MoonIcon class="hidden dark:block" />
					<span>Clair / sombre</span>
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
</Sidebar.Root>
```

Si `MenuButton` n’accepte pas `onclick` + `tooltip`, utiliser un `Button variant="ghost"` dans le footer. Ne pas ajouter de lib.

- [ ] **Step 5: Check**

Run: `bun run check`

Expected: 0 error.

- [ ] **Step 6: Commit**

```bash
git add src/app.css src/routes/+layout.svelte src/lib/components/AppSidebar.svelte package.json bun.lock
git commit -m "$(cat <<'EOF'
feat: thème pine, Geist et sidebar collapsible

EOF
)"
```

Ne pas committer `src-tauri/Cargo.toml` (diff features vides hors scope).

---

### Task 2: Primitives shadcn + EmptyState + PageHeader

**Files:**
- Create: `src/lib/components/EmptyState.svelte`, `src/lib/components/PageHeader.svelte`
- Create via CLI: `src/lib/components/ui/{select,popover,command,switch}/`
- Test: `bun run check`

**Interfaces:**
- Consumes: CLI shadcn-svelte, style mira déjà dans `components.json`
- Produces:

```ts
// EmptyState.svelte
type Props = {
	title: string;
	description: string;
	href?: string;
	actionLabel?: string;
	onclick?: () => void;
};
```

```ts
// PageHeader.svelte
import type { Snippet } from 'svelte';
type Props = { title: string; children?: Snippet };
```

- [ ] **Step 1: Ajouter les composants**

```bash
bun x shadcn-svelte@latest add select popover command switch -y
```

Si le CLI demande confirmation, passer `-y` / `--overwrite` selon l’aide. Ne pas changer `components.json` style.

- [ ] **Step 2: EmptyState**

```svelte
<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';

	type Props = {
		title: string;
		description: string;
		href?: string;
		actionLabel?: string;
		onclick?: () => void;
	};
	let { title, description, href, actionLabel, onclick }: Props = $props();
</script>

<div class="rounded-md border bg-card px-4 py-6">
	<p class="text-sm font-medium">{title}</p>
	<p class="text-muted-foreground mt-1 text-sm">{description}</p>
	{#if actionLabel && (href || onclick)}
		<Button class="mt-4" size="sm" {href} {onclick}>{actionLabel}</Button>
	{/if}
</div>
```

- [ ] **Step 3: PageHeader**

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	type Props = { title: string; children?: Snippet };
	let { title, children }: Props = $props();
</script>

<div class="flex items-center justify-between gap-4">
	<h1 class="text-lg font-semibold tracking-tight">{title}</h1>
	{#if children}
		<div class="flex items-center gap-2">{@render children()}</div>
	{/if}
</div>
```

- [ ] **Step 4: Check + commit**

`bun run check` → 0 error.

```bash
git add src/lib/components/EmptyState.svelte src/lib/components/PageHeader.svelte src/lib/components/ui package.json bun.lock
git commit -m "$(cat <<'EOF'
feat: EmptyState, PageHeader et primitives select/command

EOF
)"
```

---

### Task 3: Dashboard premier lancement + panneau RDV

**Files:**
- Modify: `src/routes/+page.svelte`
- Test: `bun run check`

**Interfaces:**
- Consumes: `rdvDashboard`, `settingsGet`, `clientsList`, `tarifsList`, `RdvPanel`, `RdvDialog`, `EmptyState`, `PageHeader`
- Produces: clic ligne → `RdvPanel` ; Jitsi seul bouton sur ligne du jour ; setup 3 étapes si 0 tarif ou 0 client.

- [ ] **Step 1: Rework `src/routes/+page.svelte`**

Charger aussi `clientsList()` et `tarifsList()`.

```ts
let panelOpen = $state(false);
let panelRdvId = $state<string | null>(null);
let loading = $state(true);
let clientsCount = $state(0);
let tarifsCount = $state(0);

const needsSetup = $derived(clientsCount === 0 || tarifsCount === 0);

async function reload() {
	loading = true;
	const [s, d, clients, tarifs] = await Promise.all([
		settingsGet(),
		rdvDashboard(),
		clientsList(),
		tarifsList()
	]);
	settings = s;
	dashboard = d;
	clientsCount = clients.length;
	tarifsCount = tarifs.length;
	loading = false;
}

function openPanel(rdv: Rdv) {
	panelRdvId = rdv.id;
	panelOpen = true;
}
```

Markup (enveloppe `p-4 flex flex-col gap-4`) :

1. `PageHeader` titre `Tableau de bord`, actions : lien Agenda (outline) + Nouveau RDV (disabled si needsSetup).
2. Si `loading` : 3 `Skeleton` `h-12`.
3. Si `needsSetup` (et pas loading) : carte avec 3 étapes (pas EmptyState unique). Texte exact :
   - « 1. Créer un tarif » bouton href `/tarifs` label `Créer un tarif` (disabled si tarifsCount > 0, alors texte muted « Fait »)
   - « 2. Créer un client » idem `/clients`
   - « 3. Créer un RDV » bouton qui ouvre `RdvDialog`, `disabled={clientsCount === 0 || tarifsCount === 0}`
4. Bandeau ntfy/Stripe existant, en dessous du setup, non bloquant. Copy : `Configuration incomplète (ntfy ou Stripe).` + lien Réglages.
5. Section `Aujourd'hui` (label uppercase spec). Lignes `divide-y`. Chaque ligne : bouton/div cliquable `openPanel(rdv)`, heure `font-mono tabular-nums`, nom, tarif muted. Un seul `Button` Jitsi `onclick` avec `stopPropagation`. Pas Copier Jitsi / Stripe sur la ligne.
6. Empty aujourd’hui : `EmptyState` title `Aucun RDV aujourd'hui` description `Créer un rendez-vous pour aujourd'hui.` actionLabel `Nouveau RDV` onclick ouvre dialog (disabled si needsSetup : alors pas de CTA, description `Créez d'abord un tarif et un client.`).
7. Section `À venir` : lignes cliquables `openPanel`, empty = `<p class="text-muted-foreground text-sm">Aucun RDV à venir.</p>`
8. Monter `RdvPanel` comme l’agenda (`bind:open`, `rdvId`, `onClose` remet `panelRdvId = null`, `onUpdated={reload}`).

- [ ] **Step 2: Check + commit**

`bun run check`

```bash
git add src/routes/+page.svelte
git commit -m "$(cat <<'EOF'
feat: dashboard setup, panneau RDV et actions Jitsi

EOF
)"
```

---

### Task 4: Agenda barre + grille horaire

**Files:**
- Modify: `src/routes/agenda/+page.svelte`, `src/lib/components/WeekGrid.svelte`, `src/lib/format.ts`
- Test: `bun run check`

**Interfaces:**
- Consumes: `Popover` + `Calendar` existant. `PageHeader`.
- Produces: `formatWeekLabel(startMonday: Date): string` dans `format.ts`. Nav Aujourd’hui / préc-suiv selon vue / popover date. Grille pine + hors plage warn.

- [ ] **Step 1: Helper `formatWeekLabel`**

Dans `src/lib/format.ts` :

```ts
export function formatWeekLabel(startMonday: Date): string {
	const end = addDays(startMonday, 6);
	const dayFmt = new Intl.DateTimeFormat('fr-FR', { day: 'numeric' });
	const endFmt = new Intl.DateTimeFormat('fr-FR', { day: 'numeric', month: 'short' });
	return `${dayFmt.format(startMonday)}–${endFmt.format(end)}`;
}

export function formatDayLabel(d: Date): string {
	return new Intl.DateTimeFormat('fr-FR', { weekday: 'short', day: 'numeric', month: 'short' }).format(d);
}
```

- [ ] **Step 2: Barre agenda**

Remplacer la rangée de boutons. Enveloppe `p-4 flex flex-col gap-4`.

```
PageHeader title Agenda
  [Aujourd’hui] [<] {dayView ? formatDayLabel(selectedDay) : formatWeekLabel(weekStart)} [>]
  segmented Semaine | Jour
  Popover calendrier
  Nouveau RDV
```

- `goToday()` : `selectedDay = new Date()`, `weekStart = startOfWeekMonday(selectedDay)`, sync `calendarValue`, `loadRdvs()`.
- `prev()` / `next()` : si `dayView`, `addDays(selectedDay, ±1)` puis `weekStart = startOfWeekMonday(selectedDay)` ; sinon `addDays(weekStart, ±7)` et `selectedDay = weekStart`. Sync calendar + load.
- Segmented : deux boutons, actif = `variant="default"`, inactif `outline`.
- Calendrier : plus en colonne. `Popover.Root` trigger bouton outline icône calendar. Content = `Calendar.Calendar` existant `type="single"` `bind:value={calendarValue}` `locale="fr-FR"`.
- Loading : `Skeleton` hauteur grille `h-[32rem]` pendant premier fetch (`let loading`).

- [ ] **Step 3: WeekGrid identité**

- Gutter heures : `font-mono tabular-nums text-muted-foreground`.
- `isToday(day)` via `sameLocalDay(day, new Date())`.
- Colonne today : `bg-primary/8` (ou `bg-primary/10`), header `border-b-2 border-primary`.
- Blocs RDV :

```svelte
class="bg-primary text-primary-foreground absolute inset-x-0.5 z-10 overflow-hidden rounded-[0.4rem] px-1 py-0.5 text-left text-xs hover:opacity-90"
```

Contenu : nom + `<span class="font-mono tabular-nums opacity-80">{formatTime(rdv.debut)}</span>`

- Hors plage : bandeau `bg-warn-bg text-warn` (classes Tailwind `bg-warn-bg` `text-warn` branchées en tâche 1). Heure visible `font-mono`. Cliquable `onRdv`.

- [ ] **Step 4: Check + commit**

`bun run check`

```bash
git add src/routes/agenda/+page.svelte src/lib/components/WeekGrid.svelte src/lib/format.ts
git commit -m "$(cat <<'EOF'
feat: barre agenda compacte et grille horaire pine

EOF
)"
```

---

### Task 5: Dialog RDV combobox client + Select tarif

**Files:**
- Create: `src/lib/components/ClientCombobox.svelte`
- Modify: `src/lib/components/RdvDialog.svelte`
- Test: `bun run check`

**Interfaces:**
- Consumes: `Popover` + `Command` (tâche 2), type `Client`
- Produces:

```ts
// ClientCombobox.svelte
type Props = {
	clients: Client[];
	value: string;
	onValueChange: (id: string) => void;
};
```

Recherche insensible à la casse sur `client.nom`. Trigger outline pleine largeur, texte = nom sélectionné ou `Choisir un client`. Si `clients.length === 0` : pas de popover, lien `<a href="/clients">Créer un client</a>`.

- [ ] **Step 1: ClientCombobox**

S’inspirer du combobox shadcn-svelte (Popover + Command). `Command.Item value={client.nom}` pour la recherche, `onSelect` appelle `onValueChange(client.id)` et ferme. Afficher le nom.

- [ ] **Step 2: RdvDialog**

Remplacer `<select>` client par `ClientCombobox bind` via `value={clientId}` `onValueChange={(id) => (clientId = id)}`.

Remplacer `<select>` tarif par `Select` shadcn. Items : option valeur `""` label `Sans tarif` + `tarifsActifs`. `onValueChange` met `tarifId` et appelle `onTarifChange`.

Garder datetime-local + durée + note.

- [ ] **Step 3: Check + commit**

`bun run check`

```bash
git add src/lib/components/ClientCombobox.svelte src/lib/components/RdvDialog.svelte
git commit -m "$(cat <<'EOF'
feat: combobox client et select tarif dans le dialog RDV

EOF
)"
```

---

### Task 6: Clients, tarifs, notes, réglages, fiche

**Files:**
- Modify: `src/routes/clients/+page.svelte`, `src/routes/clients/[id]/+page.svelte`, `src/routes/tarifs/+page.svelte`, `src/routes/notes/+page.svelte`, `src/routes/reglages/+page.svelte`
- Test: `bun run check`

**Interfaces:**
- Consumes: `EmptyState`, `PageHeader`, `Switch`, `RdvPanel`, `tarifsSetActif`, `formatDateTime`
- Produces: lignes cliquables, empty states, réactivation tarif, notes collapsed, aide réglages, historique → panneau.

- [ ] **Step 1: Clients liste**

`p-4`, `PageHeader` + Nouveau client. Skeleton si loading. Empty : `EmptyState` title `Aucun client` description `Créer une fiche pour prendre un rendez-vous.` actionLabel `Nouveau client` onclick ouvre le dialog. Sinon table, `Table.Row class="hover:bg-muted/50 cursor-pointer"` `onclick={() => goto(\`/clients/${client.id}\`)}` via `import { goto } from '$app/navigation'`. Enlever le lien seul sur le nom (toute la ligne).

- [ ] **Step 2: Tarifs**

Même chrome. Empty : title `Aucun tarif` description `Créer une prestation avec durée et prix.` actionLabel `Nouveau tarif`. Ligne cliquable → `openEdit(tarif)` (pas la colonne actions). Colonne actions : Modifier reste. Si `tarif.actif` : Désactiver. Sinon bouton `Réactiver` :

```ts
async function setActif(id: string, actif: boolean) {
	await tarifsSetActif(id, actif);
	await load();
	toast.success(actif ? 'Tarif réactivé' : 'Tarif désactivé');
}
```

Prix et durée en `font-mono tabular-nums`.

- [ ] **Step 3: Notes perso**

`PageHeader` `Notes personnelles`. Empty : title `Première note personnelle` description `Une note pour vous, hors fiche client.` pas de CTA (le champ nouvelle note suffit). Liste `divide-y` : titre = première ligne `note.corps.split('\n')[0]` tronquée 80 chars, date `formatDateTime(note.updated_at)` muted `font-mono`. Clic déplie textarea + Enregistrer / Supprimer. Une seule note dépliée à la fois (`editingId`).

- [ ] **Step 4: Réglages**

`PageHeader` `Réglages`. Aide exacte sous les champs :

- Topic : `Nom secret du canal. Sans topic, les rappels téléphone sont coupés.`
- Token : `Optionnel, si le serveur ntfy l’exige.`
- Clé Stripe : `Collez une clé secrète sk_…. Elle reste sur cette machine.` (pas de backtick dans le texte UI)

Rappels : `Switch` shadcn + Label, plus `<input type="checkbox">`. Skeleton si `!loaded`.

- [ ] **Step 5: Fiche client**

`p-4`, `PageHeader` titre = nom ou `Fiche client`. Historique : `Table.Row cursor-pointer` ouvre `RdvPanel`. Empty notes : phrase `Aucune note client.` Empty historique déjà présent, garder. Monter `RdvPanel`.

- [ ] **Step 6: Check + commit**

`bun run check`

```bash
git add src/routes/clients src/routes/tarifs src/routes/notes src/routes/reglages
git commit -m "$(cat <<'EOF'
feat: empty states, lignes cliquables et aide réglages

EOF
)"
```

---

## Spec coverage

| Spec | Task |
|---|---|
| Tokens pine / Geist / radius / padding | 1 |
| Sidebar collapse + trigger + thème | 1 |
| EmptyState / PageHeader / skeletons (composants) | 2 |
| Select popover command switch | 2 |
| Premier lancement 3 étapes | 3 |
| Dashboard panneau + Jitsi seul | 3 |
| Agenda barre / popover / grille / hors plage | 4 |
| Combobox + Select RDV | 5 |
| Clients tarifs notes réglages fiche | 6 |
| `tarifsSetActif(true)` UI | 6 |

Hors spec volontaire : lot C, backend, GSAP.
