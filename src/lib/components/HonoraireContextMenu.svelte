<script lang="ts">
	import { type Snippet } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { honorairesAnnuler, honorairesOuvrir } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Honoraire } from '$lib/types';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	type TriggerProps = Record<string, unknown>;

	type Props = {
		honoraire: Honoraire;
		children: Snippet<[TriggerProps]>;
		onUpdated?: () => void;
	};

	let { honoraire, children, onUpdated }: Props = $props();

	let cancelOpen = $state(false);
	let cancelling = $state(false);

	const isEmise = $derived(honoraire.statut === 'emise');

	async function ouvrir() {
		try {
			const path = await honorairesOuvrir(honoraire.id);
			await openPath(path);
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function confirmAnnuler() {
		cancelling = true;
		try {
			await honorairesAnnuler(honoraire.id);
			toast.success('Note d\'honoraires annulée');
			cancelOpen = false;
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
		<ContextMenu.Item onSelect={ouvrir}>Ouvrir</ContextMenu.Item>
		{#if isEmise}
			<ContextMenu.Separator />
			<ContextMenu.Item variant="destructive" onSelect={() => (cancelOpen = true)}>
				Annuler la note
			</ContextMenu.Item>
		{/if}
	</ContextMenu.Content>
</ContextMenu.Root>

<Dialog.Root bind:open={cancelOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Annuler cette note d'honoraires ?</Dialog.Title>
			<Dialog.Description>Le numéro est conservé.</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (cancelOpen = false)}>Retour</Button>
			<Button variant="destructive" onclick={confirmAnnuler} disabled={cancelling}>
				Annuler la note
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
