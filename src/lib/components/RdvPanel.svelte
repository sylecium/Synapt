<script lang="ts">
	import { onDestroy, onMount, untrack } from 'svelte';
	import { toast } from 'svelte-sonner';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import VideoIcon from '@lucide/svelte/icons/video';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import ReceiptIcon from '@lucide/svelte/icons/receipt';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { honorairesOuvrir, rdvGet, rdvSetNote, settingsGet, tarifsList } from '$lib/api';
	import type { Honoraire, RappelNtfy, Rdv, RdvDetail, SettingsPublic, Tarif } from '$lib/types';
	import { formatTarifPrix, formatTime, rdvClientLabel } from '$lib/format';
	import { noteIsEmpty } from '$lib/notesHtml';
	import { userMessage } from '$lib/errors';
	import { findHonoraireEmisForRdv } from '$lib/honoraireRdv';
	import {
		cancelRdv as cancelRdvAction,
		copyJitsi,
		copyStripe as copyStripeAction,
		openJitsi,
		openStripe as openStripeAction,
		stripeBlockedReason
	} from '$lib/rdvActions';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import NoteHtml from '$lib/components/NoteHtml.svelte';
	import RdvContextMenu from '$lib/components/RdvContextMenu.svelte';
	import HonoraireDialog from '$lib/components/HonoraireDialog.svelte';
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
	let honoraireDialogOpen = $state(false);
	let honoraireEmis = $state<Honoraire | null>(null);
	let cancelling = $state(false);
	let stripeBusy = $state(false);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let flushInFlight: Promise<void> | null = null;
	let loadedRdvId: string | null = null;
	let panelGen = 0;

	const rdv = $derived(detail?.rdv ?? null);
	const rappels = $derived(detail?.rappels ?? []);
	const cancelled = $derived(rdv?.statut === 'annule');
	const tarif = $derived(tarifs.find((t) => t.id === rdv?.tarif_id) ?? null);
	const stripeHint = $derived(rdv ? stripeBlockedReason(rdv, settings, tarifs) : '');
	const stripeDisabled = $derived(!rdv || (!rdv.stripe_url && !!stripeHint));
	const dateLabel = $derived(rdv ? formatLongDate(rdv.debut) : '');
	const timeSpan = $derived(
		rdv ? `${formatTime(rdv.debut)} - ${endTime(rdv.debut, rdv.duree_minutes)}` : ''
	);

	$effect(() => {
		const id = rdvId;
		const isOpen = open;

		untrack(() => {
			const gen = ++panelGen;
			if (isOpen && id) {
				if (loadedRdvId && loadedRdvId !== id) {
					void (async () => {
						await flushNote();
						if (gen !== panelGen) return;
						loadedRdvId = id;
						await loadDetail(id, gen);
					})();
				} else {
					loadedRdvId = id;
					void loadDetail(id, gen);
				}
			} else if (!isOpen) {
				void (async () => {
					await flushNote();
					if (gen !== panelGen) return;
					detail = null;
					loadedRdvId = null;
				})();
			}
		});
	});

	onDestroy(() => {
		void flushNote();
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

	async function loadHonoraireEmis(forRdv: Rdv, gen: number) {
		if (forRdv.statut !== 'planifie') {
			if (gen === panelGen) honoraireEmis = null;
			return;
		}
		try {
			const found = await findHonoraireEmisForRdv(forRdv.client_id, forRdv.id);
			if (gen !== panelGen) return;
			honoraireEmis = found;
		} catch (e) {
			if (gen !== panelGen) return;
			toast.error(userMessage(e));
			honoraireEmis = null;
		}
	}

	async function loadDetail(id: string, gen: number) {
		loading = true;
		honoraireEmis = null;
		try {
			const next = await rdvGet(id);
			if (gen !== panelGen) return;
			detail = next;
			void loadHonoraireEmis(next.rdv, gen);
		} catch (e) {
			if (gen !== panelGen) return;
			toast.error(userMessage(e));
			open = false;
		} finally {
			if (gen === panelGen) loading = false;
		}
	}

	function handleOpenChange(value: boolean) {
		open = value;
		if (!value) onClose();
	}

	function applyRdv(updated: Rdv) {
		detail = detail ? { ...detail, rdv: updated } : { rdv: updated, rappels: [] };
	}

	async function copyStripe() {
		if (!rdv) return;
		stripeBusy = true;
		try {
			const next = await copyStripeAction(rdv);
			if (next) applyRdv(next);
		} finally {
			stripeBusy = false;
		}
	}

	async function openStripe() {
		if (!rdv) return;
		stripeBusy = true;
		try {
			const next = await openStripeAction(rdv);
			if (next) applyRdv(next);
		} finally {
			stripeBusy = false;
		}
	}

	async function cancelRdv() {
		if (!rdv) return;
		cancelling = true;
		try {
			await cancelRdvAction(rdv.id);
			open = false;
			onUpdated();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			cancelling = false;
		}
	}

	function onNoteHtml(html: string) {
		if (!detail) return;
		detail = {
			...detail,
			rdv: { ...detail.rdv, note: noteIsEmpty(html) ? null : html }
		};
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			void flushNote();
		}, 400);
	}

	async function flushNote() {
		for (;;) {
			if (flushInFlight) {
				await flushInFlight;
				continue;
			}
			if (saveTimer) {
				clearTimeout(saveTimer);
				saveTimer = null;
			}
			const snapshot = untrack(() => {
				const current = detail?.rdv;
				if (!current || current.statut === 'annule') return null;
				return { id: current.id, note: current.note ?? null };
			});
			if (!snapshot) return;

			const run = (async () => {
				try {
					return await rdvSetNote(snapshot.id, snapshot.note);
				} catch (e) {
					toast.error(userMessage(e));
					throw e;
				}
			})();
			flushInFlight = run.then(() => {}).finally(() => {
				flushInFlight = null;
			});
			let updated: Rdv;
			try {
				updated = await run;
			} catch {
				return;
			}

			const latest = untrack(() => detail?.rdv?.note ?? null);
			if (latest !== snapshot.note) continue;
			applyRdv(updated);
			return;
		}
	}

	function rappelLabel(r: RappelNtfy): string {
		const kind = r.type === '24h' ? '24 h' : '1 h';
		return r.etat === 'programme' ? `${kind} programmé` : `${kind} annulé`;
	}

	async function openEdit() {
		await flushNote();
		editOpen = true;
	}

	function onEditSaved() {
		editOpen = false;
		if (rdvId) void loadDetail(rdvId, panelGen);
		onUpdated();
	}

	async function ouvrirHonoraire() {
		if (!honoraireEmis) return;
		try {
			const path = await honorairesOuvrir(honoraireEmis.id);
			await openPath(path);
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	function onHonoraireSaved() {
		honoraireDialogOpen = false;
		if (rdv) void loadHonoraireEmis(rdv, panelGen);
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
			<RdvContextMenu
				{rdv}
				{settings}
				{tarifs}
				showOpen={false}
				onOpen={() => {}}
				onEdit={openEdit}
				onUpdated={() => {
					if (rdvId) void loadDetail(rdvId, panelGen);
					onUpdated();
				}}
			>
				{#snippet children(props)}
					<div {...props}>
						<Sheet.Header class="gap-2 pr-12 pb-4">
							<div class="flex items-start justify-between gap-3">
								<Sheet.Title class="text-xl font-medium tracking-tight">
									{#if rdv.client_id}
										<a href="/clients/{rdv.client_id}" class="hover:underline">
											{rdvClientLabel(rdv)}
										</a>
									{:else}
										{rdvClientLabel(rdv)}
									{/if}
								</Sheet.Title>
								<Badge variant={cancelled ? 'destructive' : 'secondary'} class="mt-0.5">
									{cancelled ? 'Annulé' : 'Planifié'}
								</Badge>
							</div>
							<Sheet.Description class="text-muted-foreground text-sm capitalize">
								{dateLabel}
							</Sheet.Description>
						</Sheet.Header>
					</div>
				{/snippet}
			</RdvContextMenu>

			<div class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-6 pb-4">
				<div>
					<p class="font-mono text-2xl tracking-tight tabular-nums">{timeSpan}</p>
					<p class="text-muted-foreground mt-1 text-sm">
						{rdv.duree_minutes} min
						{#if rdv.tarif_nom}
							<span> · {rdv.tarif_nom}</span>
						{/if}
						{#if tarif && tarif.prix_centimes > 0}
							<span> · {formatTarifPrix(tarif)}</span>
						{/if}
					</p>
				</div>

				<section class="flex flex-col gap-2">
					<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">Note</p>
					{#if cancelled}
						{#if rdv.note}
							<div class="bg-muted/60 rounded-md px-3 py-2.5">
								<NoteHtml corps={rdv.note} />
							</div>
						{:else}
							<p class="text-muted-foreground text-sm">Aucune note.</p>
						{/if}
					{:else if !editOpen}
						{#key rdv.id}
							<NoteEditor
								noteId={rdv.id}
								corps={rdv.note ?? ''}
								variant="compact"
								onChange={onNoteHtml}
							/>
						{/key}
					{/if}
				</section>

				{#if !cancelled}
					<section class="flex flex-col gap-2">
						<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
							Visio
						</p>
						<div class="grid grid-cols-2 gap-2">
							<Button class="w-full" onclick={() => openJitsi(rdv)}>
								<VideoIcon />
								Ouvrir
							</Button>
							<Button variant="outline" class="w-full" onclick={() => copyJitsi(rdv)}>
								<CopyIcon />
								Copier
							</Button>
						</div>
					</section>

					{#if rdv.client_id}
						<section class="flex flex-col gap-2">
							<p class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
								Honoraires
							</p>
							<Button
								variant="outline"
								class="w-full"
								onclick={() => (honoraireEmis ? void ouvrirHonoraire() : (honoraireDialogOpen = true))}
							>
								<ReceiptIcon />
								{honoraireEmis ? 'Ouvrir la note' : "Note d'honoraires"}
							</Button>
						</section>
					{/if}

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
									disabled={stripeBusy || stripeDisabled}
								>
									<BanknoteIcon />
									{rdv.stripe_url ? 'Ouvrir' : 'Créer'}
								</Button>
								<Button
									variant="outline"
									class="w-full"
									onclick={copyStripe}
									disabled={stripeBusy || stripeDisabled}
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
						<Button variant="outline" onclick={() => void openEdit()}>Modifier</Button>
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
	{#if rdv.statut === 'planifie' && rdv.client_id}
		<HonoraireDialog
			open={honoraireDialogOpen}
			clientId={rdv.client_id}
			rdvIds={[rdv.id]}
			onClose={() => (honoraireDialogOpen = false)}
			onSaved={onHonoraireSaved}
		/>
	{/if}
{/if}
