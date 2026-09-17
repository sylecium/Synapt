<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { honorairesList, honorairesOuvrir } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Honoraire, HonoraireDetail, HonoraireStatut } from '$lib/types';
	import { formatCentimes, formatDateTime } from '$lib/format';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import HonoraireContextMenu from '$lib/components/HonoraireContextMenu.svelte';
	import HonoraireDialog from '$lib/components/HonoraireDialog.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';

	let honoraires = $state<Honoraire[]>([]);
	let initial = $state(true);
	let dialogOpen = $state(false);

	onMount(load);

	async function load() {
		try {
			honoraires = await honorairesList();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			initial = false;
		}
	}

	function statutLabel(statut: HonoraireStatut): string {
		switch (statut) {
			case 'emise':
				return 'Émise';
			case 'annulee':
				return 'Annulée';
			default: {
				const _n: never = statut;
				return _n;
			}
		}
	}

	function statutBadge(statut: HonoraireStatut): 'default' | 'secondary' | 'outline' {
		switch (statut) {
			case 'emise':
				return 'default';
			case 'annulee':
				return 'outline';
			default: {
				const _n: never = statut;
				return _n;
			}
		}
	}

	async function ouvrirHonoraire(id: string) {
		try {
			const path = await honorairesOuvrir(id);
			await openPath(path);
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	function onSaved(_detail: HonoraireDetail) {
		dialogOpen = false;
		load();
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Honoraires">
		<Button onclick={() => (dialogOpen = true)}>Nouvelle note</Button>
	</PageHeader>

	{#if initial}
		<Skeleton class="h-48" />
	{:else if honoraires.length === 0}
		<EmptyState
			title="Aucune note d'honoraires"
			description="Générez une note à partir d'une séance planifiée."
			actionLabel="Nouvelle note"
			onclick={() => (dialogOpen = true)}
		/>
	{:else}
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>Numéro</Table.Head>
					<Table.Head>Date</Table.Head>
					<Table.Head>Client</Table.Head>
					<Table.Head>Total</Table.Head>
					<Table.Head>Statut</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each honoraires as h (h.id)}
					<HonoraireContextMenu honoraire={h} onUpdated={load}>
						{#snippet children(props)}
							<Table.Row
								{...props}
								class="hover:bg-muted/50 cursor-pointer"
								onclick={() => ouvrirHonoraire(h.id)}
							>
								<Table.Cell class="font-mono tabular-nums">{h.numero}</Table.Cell>
								<Table.Cell class="font-mono tabular-nums text-sm">
									{formatDateTime(h.created_at)}
								</Table.Cell>
								<Table.Cell>{h.client_nom}</Table.Cell>
								<Table.Cell class="font-mono tabular-nums">
									{formatCentimes(h.total_centimes)}
								</Table.Cell>
								<Table.Cell>
									<Badge variant={statutBadge(h.statut)}>{statutLabel(h.statut)}</Badge>
								</Table.Cell>
							</Table.Row>
						{/snippet}
					</HonoraireContextMenu>
				{/each}
			</Table.Body>
		</Table.Root>
	{/if}
</div>

<HonoraireDialog
	open={dialogOpen}
	onClose={() => (dialogOpen = false)}
	onSaved={onSaved}
/>
