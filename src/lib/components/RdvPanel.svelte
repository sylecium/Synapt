<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import VideoIcon from '@lucide/svelte/icons/video';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import { rdvAnnuler, rdvGet, settingsGet, stripeEnsureLink, tarifsList } from '$lib/api';
	import type { RappelNtfy, Rdv, RdvDetail, SettingsPublic, Tarif } from '$lib/types';
	import { formatCentimes, formatTime } from '$lib/format';
	import { userMessage } from '$lib/errors';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Sheet from '$lib/components/ui/sheet/index.js';

	type Props = {
		open: boolean;
		rdvId: string | null;
		onClose: () => void;
		onUpdated: () => void;
	};

	let { open = $bindable(), rdvId, onClose, onUpdated }: Props = $props();

	let detail = $state<RdvDetail | null>(null);
	let settings = $state<SettingsPublic | null>(null);
	let tarifs = $state<Tarif[]>([]);
	let loading = $state(false);
	let editOpen = $state(false);
	let cancelling = $state(false);
	let stripeBusy = $state(false);

	const rdv = $derived(detail?.rdv ?? null);
	const rappels = $derived(detail?.rappels ?? []);
	const cancelled = $derived(rdv?.statut === 'annule');
	const tarif = $derived(tarifs.find((t) => t.id === rdv?.tarif_id) ?? null);
	const stripeDisabled = $derived.by(() => {
		if (!rdv) return true;
		if (rdv.stripe_url) return false;
		if (!settings?.stripe_configured) return true;
		if (!rdv.tarif_id) return true;
		return !tarif || tarif.prix_centimes === 0;
	});
	const stripeHint = $derived.by(() => {
		if (!rdv || rdv.stripe_url) return '';
		if (!settings?.stripe_configured) return "Stripe n'est pas configuré.";
		if (!rdv.tarif_id) return 'Ajoutez un tarif pour demander un paiement.';
		if (tarif && tarif.prix_centimes === 0) return 'Ce tarif est à 0 €.';
		return '';
	});
	const dateLabel = $derived(rdv ? formatLongDate(rdv.debut) : '');
	const timeSpan = $derived(
		rdv ? `${formatTime(rdv.debut)} – ${endTime(rdv.debut, rdv.duree_minutes)}` : ''
	);

	$effect(() => {
		if (open && rdvId) {
			loadDetail(rdvId);
		} else if (!open) {
			detail = null;
		}
	});

	onMount(async () => {
		[settings, tarifs] = await Promise.all([settingsGet(), tarifsList()]);
	});

	function formatLongDate(iso: string): string {
		return new Intl.DateTimeFormat('fr-FR', {
			weekday: 'long',
			day: 'numeric',
			month: 'long'
		}).format(new Date(iso));
	}

	function endTime(iso: string, minutes: number): string {
		const d = new Date(iso);
		d.setMinutes(d.getMinutes() + minutes);
		return formatTime(d.toISOString());
	}

	async function loadDetail(id: string) {
		loading = true;
		try {
			detail = await rdvGet(id);
		} catch (e) {
			toast.error(userMessage(e));
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
			toast.error(userMessage(e));
		}
	}

	async function openLink(url: string) {
		try {
			await openUrl(url);
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function ensureStripeLink(current: Rdv): Promise<Rdv> {
		if (current.stripe_url) return current;
		stripeBusy = true;
		try {
			const updated = await stripeEnsureLink(current.id);
			detail = detail ? { ...detail, rdv: updated } : { rdv: updated, rappels: [] };
			return updated;
		} catch (e) {
			toast.error(userMessage(e));
			throw e;
		} finally {
			stripeBusy = false;
		}
	}

	async function copyStripe() {
		if (!rdv) return;
		const current = await ensureStripeLink(rdv);
		if (current.stripe_url) await copyUrl(current.stripe_url, 'Lien de paiement');
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
			const result = await rdvAnnuler(rdv.id);
			for (const w of result.warnings) {
				toast.error(userMessage(w));
			}
			toast.success('RDV annulé');
			open = false;
			onUpdated();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			cancelling = false;
		}
	}

	function rappelLabel(r: RappelNtfy): string {
		const kind = r.type === '24h' ? '24 h' : '1 h';
		return r.etat === 'programme' ? `${kind} programmé` : `${kind} annulé`;
	}

	function onEditSaved() {
		editOpen = false;
		if (rdvId) loadDetail(rdvId);
		onUpdated();
	}
</script>

<Sheet.Root open={open} onOpenChange={handleOpenChange}>
	<Sheet.Content side="right" class="w-full p-0 text-sm sm:max-w-md">
		{#if loading}
			<Sheet.Header class="pr-12">
				<Skeleton class="h-7 w-40" />
				<Skeleton class="h-4 w-56" />
			</Sheet.Header>
			<div class="flex flex-col gap-3 px-6">
				<Skeleton class="h-16" />
				<Skeleton class="h-24" />
				<Skeleton class="h-24" />
			</div>
		{:else if rdv}
			<Sheet.Header class="gap-2 pr-12 pb-4">
				<div class="flex items-start justify-between gap-3">
					<Sheet.Title class="text-xl font-medium tracking-tight">
						<a href="/clients/{rdv.client_id}" class="hover:underline">
							{rdv.client_nom}
						</a>
					</Sheet.Title>
					<Badge variant={cancelled ? 'destructive' : 'secondary'} class="mt-0.5">
						{cancelled ? 'Annulé' : 'Planifié'}
					</Badge>
				</div>
				<Sheet.Description class="text-muted-foreground text-sm capitalize">
					{dateLabel}
				</Sheet.Description>
			</Sheet.Header>

			<div class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-6 pb-4">
				<div>
					<p class="font-mono text-2xl tracking-tight tabular-nums">{timeSpan}</p>
					<p class="text-muted-foreground mt-1 text-sm">
						{rdv.duree_minutes} min
						{#if rdv.tarif_nom}
							<span> · {rdv.tarif_nom}</span>
						{/if}
						{#if tarif && tarif.prix_centimes > 0}
							<span> · {formatCentimes(tarif.prix_centimes)}</span>
						{/if}
					</p>
				</div>

				{#if rdv.note}
					<div class="bg-muted/60 rounded-md px-3 py-2.5">
						<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
							Note
						</p>
						<p class="mt-1 whitespace-pre-wrap leading-relaxed">{rdv.note}</p>
					</div>
				{/if}

				{#if !cancelled}
					<section class="flex flex-col gap-2">
						<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
							Visio
						</p>
						<div class="grid grid-cols-2 gap-2">
							<Button class="w-full" onclick={() => openLink(rdv.jitsi_url)}>
								<VideoIcon />
								Ouvrir
							</Button>
							<Button
								variant="outline"
								class="w-full"
								onclick={() => copyUrl(rdv.jitsi_url, 'Lien Jitsi')}
							>
								<CopyIcon />
								Copier
							</Button>
						</div>
					</section>

					<section class="flex flex-col gap-2">
						<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
							Paiement
						</p>
						{#if stripeHint}
							<p class="text-muted-foreground text-sm">{stripeHint}</p>
						{:else}
							<div class="grid grid-cols-2 gap-2">
								<Button
									variant="outline"
									class="w-full"
									onclick={openStripe}
									disabled={stripeBusy}
								>
									<BanknoteIcon />
									{rdv.stripe_url ? 'Ouvrir' : 'Créer'}
								</Button>
								<Button
									variant="outline"
									class="w-full"
									onclick={copyStripe}
									disabled={stripeBusy}
								>
									<CopyIcon />
									Copier
								</Button>
							</div>
						{/if}
					</section>
				{/if}

				{#if rappels.length > 0}
					<section class="flex flex-col gap-2">
						<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
							Rappels
						</p>
						<ul class="flex flex-wrap gap-1.5">
							{#each rappels as r (r.id)}
								<li>
									<Badge variant={r.etat === 'programme' ? 'outline' : 'ghost'}>
										{rappelLabel(r)}
									</Badge>
								</li>
							{/each}
						</ul>
					</section>
				{/if}
			</div>

			<Sheet.Footer class="border-t">
				{#if cancelled}
					<p class="text-muted-foreground text-sm">Ce rendez-vous est annulé.</p>
				{:else}
					<div class="grid grid-cols-2 gap-2">
						<Button variant="outline" onclick={() => (editOpen = true)}>Modifier</Button>
						<Button variant="destructive" onclick={cancelRdv} disabled={cancelling}>
							Annuler
						</Button>
					</div>
				{/if}
			</Sheet.Footer>
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
