<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { rdvDashboard, settingsGet } from '$lib/api';
	import type { Dashboard, Rdv, RdvCreateResult, SettingsPublic } from '$lib/types';
	import { formatDateTime, formatTime } from '$lib/format';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';

	let dashboard = $state<Dashboard | null>(null);
	let settings = $state<SettingsPublic | null>(null);
	let rdvDialogOpen = $state(false);

	const showBanner = $derived(
		settings !== null && (!settings.ntfy_topic || !settings.stripe_configured)
	);

	onMount(async () => {
		await reload();
	});

	async function reload() {
		[settings, dashboard] = await Promise.all([settingsGet(), rdvDashboard()]);
	}

	async function copyUrl(url: string, label: string) {
		try {
			await writeText(url);
			toast.success(`${label} copié`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function openLink(url: string) {
		try {
			await openUrl(url);
		} catch (e) {
			toast.error(String(e));
		}
	}

	function onRdvSaved(_result: RdvCreateResult) {
		rdvDialogOpen = false;
		reload();
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Tableau de bord</h1>
		<div class="flex gap-2">
			<Button variant="outline" href="/agenda">Agenda</Button>
			<Button onclick={() => (rdvDialogOpen = true)}>Nouveau RDV</Button>
		</div>
	</div>

	{#if showBanner}
		<div class="bg-muted rounded-md border px-4 py-3 text-sm">
			Configuration incomplète (ntfy ou Stripe).
			<a href="/reglages" class="text-primary underline">Réglages</a>
		</div>
	{/if}

	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium">Aujourd'hui</h2>
		{#if dashboard?.aujourdhui.length}
			<ul class="flex flex-col gap-3">
				{#each dashboard.aujourdhui as rdv (rdv.id)}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-md border px-4 py-3">
						<span class="w-14 font-medium tabular-nums">{formatTime(rdv.debut)}</span>
						<a href="/clients/{rdv.client_id}" class="hover:underline">{rdv.client_nom}</a>
						<span class="text-muted-foreground text-sm">{rdv.tarif_nom || '—'}</span>
						<div class="ml-auto flex flex-wrap gap-2">
							<Button
								variant="outline"
								size="sm"
								onclick={() => openLink(rdv.jitsi_url)}
							>
								Jitsi
							</Button>
							<Button
								variant="outline"
								size="sm"
								onclick={() => copyUrl(rdv.jitsi_url, 'Lien Jitsi')}
							>
								Copier Jitsi
							</Button>
							{#if rdv.stripe_url}
								<Button
									variant="outline"
									size="sm"
									onclick={() => openLink(rdv.stripe_url!)}
								>
									Stripe
								</Button>
							{/if}
						</div>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-muted-foreground text-sm">Aucun RDV aujourd'hui.</p>
		{/if}
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium">À venir</h2>
		{#if dashboard?.a_venir.length}
			<ul class="flex flex-col gap-2">
				{#each dashboard.a_venir as rdv (rdv.id)}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
						<span class="text-muted-foreground w-36">{formatDateTime(rdv.debut)}</span>
						<a href="/clients/{rdv.client_id}" class="hover:underline">{rdv.client_nom}</a>
						<span class="text-muted-foreground">{rdv.tarif_nom || '—'}</span>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-muted-foreground text-sm">Aucun RDV à venir.</p>
		{/if}
	</section>
</div>

<RdvDialog
	open={rdvDialogOpen}
	onClose={() => (rdvDialogOpen = false)}
	onSaved={onRdvSaved}
/>
