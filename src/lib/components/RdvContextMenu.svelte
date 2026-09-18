<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { honorairesOuvrir, settingsGet, tarifsList } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import { findHonoraireEmisForRdv } from '$lib/honoraireRdv';
	import HonoraireDialog from '$lib/components/HonoraireDialog.svelte';
	import type { Honoraire } from '$lib/types';
	import {
		cancelRdv,
		copyJitsi,
		copyStripe,
		openJitsi,
		openStripe,
		stripeBlockedReason
	} from '$lib/rdvActions';
	import type { Rdv, SettingsPublic, Tarif } from '$lib/types';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	type TriggerProps = Record<string, unknown>;

	type Props = {
		rdv: Rdv;
		children: Snippet<[TriggerProps]>;
		onOpen: () => void;
		onEdit?: () => void;
		onUpdated?: () => void;
		showOpen?: boolean;
		settings?: SettingsPublic | null;
		tarifs?: Tarif[];
	};

	let {
		rdv,
		children,
		onOpen,
		onEdit,
		onUpdated,
		showOpen = true,
		settings: settingsFromParent,
		tarifs: tarifsFromParent
	}: Props = $props();

	let settingsLocal = $state<SettingsPublic | null>(null);
	let tarifsLocal = $state<Tarif[]>([]);
	let stripeBusy = $state(false);
	let cancelling = $state(false);
	let honoraireEmis = $state<Honoraire | null>(null);
	let honoraireLoading = $state(false);
	let honoraireDialogOpen = $state(false);

	const settings = $derived(settingsFromParent ?? settingsLocal);
	const tarifs = $derived(tarifsFromParent ?? tarifsLocal);
	const cancelled = $derived(rdv.statut === 'annule');
	const planifie = $derived(rdv.statut === 'planifie');
	const stripeHint = $derived(stripeBlockedReason(rdv, settings, tarifs));
	const stripeDisabled = $derived(!!stripeHint);

	onMount(async () => {
		if (settingsFromParent !== undefined) return;
		try {
			[settingsLocal, tarifsLocal] = await Promise.all([settingsGet(), tarifsList()]);
		} catch (e) {
			toast.error(userMessage(e));
		}
	});

	async function onStripeOpen() {
		stripeBusy = true;
		try {
			const next = await openStripe(rdv);
			if (next) onUpdated?.();
		} finally {
			stripeBusy = false;
		}
	}

	async function onStripeCopy() {
		stripeBusy = true;
		try {
			const next = await copyStripe(rdv);
			if (next) onUpdated?.();
		} finally {
			stripeBusy = false;
		}
	}

	async function onCancel() {
		cancelling = true;
		try {
			await cancelRdv(rdv.id);
			onUpdated?.();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			cancelling = false;
		}
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

	async function refreshHonoraireEmis() {
		if (!planifie || !rdv.client_id) {
			honoraireEmis = null;
			honoraireLoading = false;
			return;
		}
		honoraireLoading = true;
		try {
			honoraireEmis = await findHonoraireEmisForRdv(rdv.client_id, rdv.id);
		} catch (e) {
			toast.error(userMessage(e));
			honoraireEmis = null;
		} finally {
			honoraireLoading = false;
		}
	}

	function onHonoraireSaved() {
		honoraireDialogOpen = false;
		void refreshHonoraireEmis();
		onUpdated?.();
	}
</script>

<ContextMenu.Root
	onOpenChange={(open) => {
		if (open) {
			void refreshHonoraireEmis();
		}
	}}
>
	<ContextMenu.Trigger>
		{#snippet child({ props })}
			{@render children(props)}
		{/snippet}
	</ContextMenu.Trigger>
	<ContextMenu.Content class="w-52">
		{#if showOpen}
			<ContextMenu.Item onSelect={onOpen}>Ouvrir</ContextMenu.Item>
		{/if}
		<ContextMenu.Item disabled={cancelled} onSelect={() => openJitsi(rdv)}>
			Ouvrir Jitsi
		</ContextMenu.Item>
		<ContextMenu.Item disabled={cancelled} onSelect={() => copyJitsi(rdv)}>
			Copier le lien Jitsi
		</ContextMenu.Item>
		<ContextMenu.Separator />
		<ContextMenu.Item
			disabled={cancelled || stripeDisabled || stripeBusy}
			onSelect={onStripeOpen}
		>
			{rdv.stripe_url ? 'Ouvrir Stripe' : 'Créer le lien Stripe'}
		</ContextMenu.Item>
		<ContextMenu.Item
			disabled={cancelled || stripeDisabled || stripeBusy}
			onSelect={onStripeCopy}
		>
			Copier le lien Stripe
		</ContextMenu.Item>
		{#if planifie && rdv.client_id}
			<ContextMenu.Separator />
			{#if honoraireEmis}
				<ContextMenu.Item onSelect={ouvrirHonoraire}>Ouvrir la note</ContextMenu.Item>
			{:else if !honoraireLoading}
				<ContextMenu.Item onSelect={() => (honoraireDialogOpen = true)}>
					Note d'honoraires
				</ContextMenu.Item>
			{/if}
		{/if}
		<ContextMenu.Separator />
		<ContextMenu.Item disabled={cancelled} onSelect={() => onEdit?.()}>
			Modifier
		</ContextMenu.Item>
		<ContextMenu.Item
			variant="destructive"
			disabled={cancelled || cancelling}
			onSelect={onCancel}
		>
			Annuler le RDV
		</ContextMenu.Item>
	</ContextMenu.Content>
</ContextMenu.Root>

{#if planifie && rdv.client_id}
	<HonoraireDialog
		open={honoraireDialogOpen}
		clientId={rdv.client_id}
		rdvIds={[rdv.id]}
		onClose={() => (honoraireDialogOpen = false)}
		onSaved={onHonoraireSaved}
	/>
{/if}
