<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { rdvAnnuler, rdvGet, stripeEnsureLink } from '$lib/api';
	import type { RappelNtfy, Rdv, RdvDetail } from '$lib/types';
	import { formatDateTime } from '$lib/format';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Sheet from '$lib/components/ui/sheet/index.js';

	type Props = {
		open: boolean;
		rdvId: string | null;
		onClose: () => void;
		onUpdated: () => void;
	};

	let { open = $bindable(), rdvId, onClose, onUpdated }: Props = $props();

	let detail = $state<RdvDetail | null>(null);
	let loading = $state(false);
	let editOpen = $state(false);
	let cancelling = $state(false);

	const rdv = $derived(detail?.rdv ?? null);
	const rappels = $derived(detail?.rappels ?? []);

	$effect(() => {
		if (open && rdvId) {
			loadDetail(rdvId);
		} else if (!open) {
			detail = null;
		}
	});

	async function loadDetail(id: string) {
		loading = true;
		try {
			detail = await rdvGet(id);
		} catch (e) {
			toast.error(String(e));
			open = false;
		} finally {
			loading = false;
		}
	}

	function handleOpenChange(value: boolean) {
		open = value;
		if (!value) onClose();
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

	async function ensureStripeLink(current: Rdv): Promise<Rdv> {
		if (current.stripe_url) return current;
		try {
			const updated = await stripeEnsureLink(current.id);
			detail = detail ? { ...detail, rdv: updated } : { rdv: updated, rappels: [] };
			return updated;
		} catch (e) {
			toast.error(String(e));
			throw e;
		}
	}

	async function copyStripe() {
		if (!rdv) return;
		const current = await ensureStripeLink(rdv);
		if (current.stripe_url) await copyUrl(current.stripe_url, 'Lien Stripe');
	}

	async function openStripe() {
		if (!rdv) return;
		const current = await ensureStripeLink(rdv);
		if (current.stripe_url) await openLink(current.stripe_url);
	}

	async function cancelRdv() {
		if (!rdv) return;
		cancelling = true;
		try {
			await rdvAnnuler(rdv.id);
			toast.success('RDV annulé');
			open = false;
			onUpdated();
		} catch (e) {
			toast.error(String(e));
		} finally {
			cancelling = false;
		}
	}

	function rappelLabel(r: RappelNtfy): string {
		const kind = r.type === '24h' ? '24 h' : '1 h';
		const etat = r.etat === 'programme' ? 'programmé' : 'annulé';
		return `Rappel ${kind} : ${etat}`;
	}

	function onEditSaved() {
		editOpen = false;
		if (rdvId) loadDetail(rdvId);
		onUpdated();
	}
</script>

<Sheet.Root open={open} onOpenChange={handleOpenChange}>
	<Sheet.Content side="right" class="w-full sm:max-w-md">
		<Sheet.Header>
			<Sheet.Title>RDV</Sheet.Title>
		</Sheet.Header>

		{#if loading}
			<p class="text-muted-foreground text-sm">Chargement…</p>
		{:else if rdv}
			<div class="flex flex-col gap-4 py-4">
				<div>
					<p class="text-muted-foreground text-xs">Client</p>
					<p class="font-medium">{rdv.client_nom}</p>
				</div>
				<div>
					<p class="text-muted-foreground text-xs">Horaire</p>
					<p>{formatDateTime(rdv.debut)} · {rdv.duree_minutes} min</p>
					{#if rdv.tarif_nom}
						<p class="text-muted-foreground text-sm">{rdv.tarif_nom}</p>
					{/if}
				</div>
				{#if rdv.note}
					<div>
						<p class="text-muted-foreground text-xs">Note</p>
						<p class="text-sm whitespace-pre-wrap">{rdv.note}</p>
					</div>
				{/if}
				{#if rappels.length > 0}
					<div>
						<p class="text-muted-foreground text-xs">Rappels ntfy</p>
						<ul class="text-sm">
							{#each rappels as r (r.id)}
								<li>{rappelLabel(r)}</li>
							{/each}
						</ul>
					</div>
				{/if}

				<div class="flex flex-wrap gap-2">
					<Button variant="outline" size="sm" onclick={() => copyUrl(rdv.jitsi_url, 'Lien Jitsi')}>
						Copier Jitsi
					</Button>
					<Button variant="outline" size="sm" onclick={() => openLink(rdv.jitsi_url)}>
						Ouvrir Jitsi
					</Button>
					<Button variant="outline" size="sm" onclick={copyStripe}>Copier Stripe</Button>
					<Button variant="outline" size="sm" onclick={openStripe}>Ouvrir Stripe</Button>
				</div>

				<div class="flex flex-wrap gap-2 pt-2">
					<Button variant="outline" onclick={() => (editOpen = true)}>Modifier</Button>
					<Button variant="destructive" onclick={cancelRdv} disabled={cancelling}>
						Annuler le RDV
					</Button>
				</div>
			</div>
		{/if}
	</Sheet.Content>
</Sheet.Root>

{#if rdv}
	<RdvDialog
		open={editOpen}
		rdvId={rdv.id}
		onClose={() => (editOpen = false)}
		onSaved={onEditSaved}
	/>
{/if}
