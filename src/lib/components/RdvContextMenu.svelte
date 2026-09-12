<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { settingsGet, tarifsList } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import {
		cancelRdv,
		copyJitsi,
		copyStripe,
		openJitsi,
		openStripe,
		stripeBlockedReason
	} from '$lib/rdvActions';
	import type { Rdv, SettingsPublic, Tarif } from '$lib/types';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	type TriggerProps = Record<string, unknown>;

	type Props = {
		rdv: Rdv;
		children: Snippet<[TriggerProps]>;
		onOpen: () => void;
		onUpdated?: () => void;
		showOpen?: boolean;
	};

	let { rdv, children, onOpen, onUpdated, showOpen = true }: Props = $props();

	let settings = $state<SettingsPublic | null>(null);
	let tarifs = $state<Tarif[]>([]);
	let editOpen = $state(false);
	let stripeBusy = $state(false);
	let cancelling = $state(false);

	const cancelled = $derived(rdv.statut === 'annule');
	const stripeHint = $derived(stripeBlockedReason(rdv, settings, tarifs));
	const stripeDisabled = $derived(!!stripeHint);

	onMount(async () => {
		try {
			[settings, tarifs] = await Promise.all([settingsGet(), tarifsList()]);
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
</script>

<ContextMenu.Root>
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
		<ContextMenu.Separator />
		<ContextMenu.Item
			disabled={cancelled}
			onSelect={() => {
				editOpen = true;
			}}
		>
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

<RdvDialog
	open={editOpen}
	rdvId={rdv.id}
	onClose={() => (editOpen = false)}
	onSaved={() => {
		editOpen = false;
		onUpdated?.();
	}}
/>
