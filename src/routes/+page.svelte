<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { rdvDashboard, settingsGet } from '$lib/api';
	import { nextRdv, remainingRdvs } from '$lib/dashboardStats';
	import { userMessage } from '$lib/errors';
	import type { Dashboard, Rdv, RdvCreateResult, SettingsPublic } from '$lib/types';
	import {
		formatDateTime,
		formatTime,
		rdvClientLabel,
		sameLocalDay
	} from '$lib/format';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import RdvContextMenu from '$lib/components/RdvContextMenu.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import RdvPanel from '$lib/components/RdvPanel.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { openJitsi } from '$lib/rdvActions';

	let dashboard = $state<Dashboard | null>(null);
	let settings = $state<SettingsPublic | null>(null);
	let weekCount = $state(0);
	let rdvDialogOpen = $state(false);
	let panelOpen = $state(false);
	let panelRdvId = $state<string | null>(null);
	let editRdvId = $state<string | null>(null);
	let initial = $state(true);
	let clientsCount = $state(0);
	let tarifsCount = $state(0);
	let now = $state(new Date());

	const needsSetup = $derived(
		dashboard !== null && (clientsCount === 0 || tarifsCount === 0)
	);

	const showBanner = $derived(
		settings !== null && (!settings.ntfy_topic || !settings.stripe_configured)
	);

	const restants = $derived(
		dashboard ? remainingRdvs(dashboard.aujourdhui, now).length : 0
	);

	const prochain = $derived(
		dashboard ? nextRdv(dashboard.aujourdhui, dashboard.a_venir, now) : null
	);

	onMount(() => {
		reload();
		const id = setInterval(() => {
			now = new Date();
		}, 60_000);
		return () => clearInterval(id);
	});

	async function reload() {
		try {
			const [s, d] = await Promise.all([
				settingsGet(),
				rdvDashboard()
			]);
			settings = s;
			dashboard = d;
			clientsCount = d.clients_count;
			tarifsCount = d.tarifs_count;
			weekCount = d.week_count;
			now = new Date();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			initial = false;
		}
	}

	function openPanel(rdv: Rdv) {
		panelRdvId = rdv.id;
		panelOpen = true;
	}

	function openEditRdv(r: Rdv) {
		editRdvId = r.id;
		rdvDialogOpen = true;
	}

	function onRdvSaved(_result: RdvCreateResult) {
		rdvDialogOpen = false;
		editRdvId = null;
		reload();
	}

	function prochainHoraire(rdv: Rdv): string {
		return sameLocalDay(new Date(rdv.debut), now) ? formatTime(rdv.debut) : formatDateTime(rdv.debut);
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Tableau de bord">
		<Button variant="outline" href="/agenda">Agenda</Button>
		<Button disabled={needsSetup} onclick={() => (rdvDialogOpen = true)}>Nouveau RDV</Button>
	</PageHeader>

	{#if initial}
		<Skeleton class="h-20" />
		<Skeleton class="h-24" />
		<Skeleton class="h-12" />
		<Skeleton class="h-12" />
	{:else if dashboard === null}
		<p class="text-muted-foreground text-sm">Impossible de charger le tableau de bord.</p>
	{:else}
		{#if needsSetup}
			<div class="rounded-lg border bg-card px-4 py-3">
				<div class="flex flex-col divide-y">
					<div class="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
						<span class="text-sm">1. Créer un tarif</span>
						{#if tarifsCount > 0}
							<span class="text-muted-foreground text-sm">Fait</span>
						{:else}
							<Button size="sm" href="/tarifs">Créer un tarif</Button>
						{/if}
					</div>
					<div class="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
						<span class="text-sm">2. Créer un client</span>
						{#if clientsCount > 0}
							<span class="text-muted-foreground text-sm">Fait</span>
						{:else}
							<Button size="sm" href="/clients">Créer un client</Button>
						{/if}
					</div>
					<div class="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
						<span class="text-sm">3. Créer un RDV</span>
						<Button
							size="sm"
							disabled={clientsCount === 0 || tarifsCount === 0}
							onclick={() => (rdvDialogOpen = true)}
						>
							Créer un RDV
						</Button>
					</div>
				</div>
			</div>
		{/if}

		{#if showBanner}
			<div class="bg-muted rounded-lg border px-4 py-3 text-sm">
				Configuration incomplète (ntfy ou Stripe).
				<a href="/reglages" class="text-primary underline">Réglages</a>
			</div>
		{/if}

		<div class="grid grid-cols-3 divide-x rounded-lg border">
			<div class="px-4 py-3">
				<p class="font-mono text-2xl tabular-nums tracking-tight">{dashboard.aujourdhui.length}</p>
				<p class="text-muted-foreground mt-0.5 text-xs font-medium tracking-wide uppercase">
					Aujourd'hui
				</p>
			</div>
			<div class="px-4 py-3">
				<p class="font-mono text-2xl tabular-nums tracking-tight">{restants}</p>
				<p class="text-muted-foreground mt-0.5 text-xs font-medium tracking-wide uppercase">
					Restants
				</p>
			</div>
			<div class="px-4 py-3">
				<p class="font-mono text-2xl tabular-nums tracking-tight">{weekCount}</p>
				<p class="text-muted-foreground mt-0.5 text-xs font-medium tracking-wide uppercase">
					Cette semaine
				</p>
			</div>
		</div>

		{#if prochain}
			<section class="flex flex-col gap-2">
				<h2 class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Prochain</h2>
				<RdvContextMenu
					rdv={prochain}
					onOpen={() => openPanel(prochain)}
					onEdit={() => openEditRdv(prochain)}
					onUpdated={reload}
				>
					{#snippet children(props)}
						<div {...props} class="flex items-center gap-3 rounded-lg border px-4 py-3">
							<button
								type="button"
								class="flex min-w-0 flex-1 items-center gap-4 text-left"
								onclick={() => openPanel(prochain)}
							>
								<span class="font-mono text-xl tabular-nums tracking-tight">
									{prochainHoraire(prochain)}
								</span>
								<span class="min-w-0">
									<span class="block truncate font-medium">{rdvClientLabel(prochain)}</span>
									<span class="text-muted-foreground block truncate text-sm">
										{prochain.tarif_nom || '-'} · {prochain.duree_minutes} min
									</span>
								</span>
							</button>
							<Button
								size="sm"
								onclick={(e) => {
									e.stopPropagation();
									void openJitsi(prochain);
								}}
							>
								Jitsi
							</Button>
						</div>
					{/snippet}
				</RdvContextMenu>
			</section>
		{/if}

		<section class="flex flex-col gap-3">
			<h2 class="text-xs font-medium tracking-wide text-muted-foreground uppercase">
				Aujourd'hui
			</h2>
			{#if dashboard.aujourdhui.length}
				<ul class="divide-y rounded-lg border">
					{#each dashboard.aujourdhui as rdv (rdv.id)}
						<li>
							<RdvContextMenu
								{rdv}
								onOpen={() => openPanel(rdv)}
								onEdit={() => openEditRdv(rdv)}
								onUpdated={reload}
							>
								{#snippet children(props)}
									<div {...props} class="flex items-center gap-x-4 px-4 py-3">
										<button
											type="button"
											class="flex min-w-0 flex-1 items-center gap-x-4 text-left"
											onclick={() => openPanel(rdv)}
										>
											<span class="w-14 font-mono tabular-nums">{formatTime(rdv.debut)}</span>
											<span>{rdvClientLabel(rdv)}</span>
											<span class="text-muted-foreground text-sm">{rdv.tarif_nom || '-'}</span>
										</button>
										<Button
											size="sm"
											onclick={(e) => {
												e.stopPropagation();
												void openJitsi(rdv);
											}}
										>
											Jitsi
										</Button>
									</div>
								{/snippet}
							</RdvContextMenu>
						</li>
					{/each}
				</ul>
			{:else}
				<EmptyState
					title="Aucun RDV aujourd'hui"
					description={needsSetup
						? "Créez d'abord un tarif et un client."
						: "Créer un rendez-vous pour aujourd'hui."}
					actionLabel={needsSetup ? undefined : 'Nouveau RDV'}
					onclick={needsSetup ? undefined : () => (rdvDialogOpen = true)}
				/>
			{/if}
		</section>

		<section class="flex flex-col gap-3">
			<h2 class="text-xs font-medium tracking-wide text-muted-foreground uppercase">À venir</h2>
			{#if dashboard.a_venir.length}
				<ul class="divide-y rounded-lg border">
					{#each dashboard.a_venir as rdv (rdv.id)}
						<li>
							<RdvContextMenu
								{rdv}
								onOpen={() => openPanel(rdv)}
								onEdit={() => openEditRdv(rdv)}
								onUpdated={reload}
							>
								{#snippet children(props)}
									<button
										{...props}
										type="button"
										class="flex w-full items-center gap-x-4 px-4 py-3 text-left text-sm"
										onclick={() => openPanel(rdv)}
									>
										<span class="text-muted-foreground w-36 font-mono tabular-nums">
											{formatDateTime(rdv.debut)}
										</span>
										<span>{rdvClientLabel(rdv)}</span>
										<span class="text-muted-foreground">{rdv.tarif_nom || '-'}</span>
									</button>
								{/snippet}
							</RdvContextMenu>
						</li>
					{/each}
				</ul>
			{:else}
				<p class="text-muted-foreground text-sm">Aucun RDV à venir.</p>
			{/if}
		</section>
	{/if}
</div>

<RdvDialog
	open={rdvDialogOpen}
	rdvId={editRdvId ?? undefined}
	onClose={() => {
		rdvDialogOpen = false;
		editRdvId = null;
	}}
	onSaved={onRdvSaved}
/>

<RdvPanel
	bind:open={panelOpen}
	rdvId={panelRdvId}
	onClose={() => {
		panelRdvId = null;
	}}
	onUpdated={reload}
/>
