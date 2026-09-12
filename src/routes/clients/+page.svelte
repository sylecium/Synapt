<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { clientsList, clientsUpsert } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Client } from '$lib/types';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	let clients = $state<Client[]>([]);
	let recherche = $state('');
	let dialogOpen = $state(false);
	let nom = $state('');
	let email = $state('');
	let telephone = $state('');
	let saving = $state(false);
	let initial = $state(true);
	let rdvDialogOpen = $state(false);
	let rdvClientId = $state<string | undefined>();

	const filtered = $derived(
		clients.filter((c) => c.nom.toLowerCase().includes(recherche.toLowerCase()))
	);

	onMount(load);

	async function load() {
		try {
			clients = await clientsList();
		} finally {
			initial = false;
		}
	}

	function openCreate() {
		nom = '';
		email = '';
		telephone = '';
		dialogOpen = true;
	}

	async function save() {
		if (!nom.trim()) {
			toast.error('Le nom est requis');
			return;
		}
		saving = true;
		try {
			await clientsUpsert({
				nom: nom.trim(),
				email: email.trim() || null,
				telephone: telephone.trim() || null
			});
			dialogOpen = false;
			await load();
			toast.success('Client créé');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			saving = false;
		}
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Clients">
		<Button onclick={openCreate}>Nouveau client</Button>
	</PageHeader>

	{#if initial}
		<Skeleton class="h-10 max-w-sm" />
		<Skeleton class="h-48" />
	{:else if clients.length === 0}
		<EmptyState
			title="Aucun client"
			description="Créer une fiche pour prendre un rendez-vous."
			actionLabel="Nouveau client"
			onclick={openCreate}
		/>
	{:else}
		<Input placeholder="Rechercher par nom…" bind:value={recherche} class="max-w-sm" />

		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>Nom</Table.Head>
					<Table.Head>Email</Table.Head>
					<Table.Head>Téléphone</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each filtered as client (client.id)}
					<ContextMenu.Root>
						<ContextMenu.Trigger>
							{#snippet child({ props })}
								<Table.Row class="hover:bg-muted/50 relative" {...props}>
									<Table.Cell class="font-medium">
										<a href="/clients/{client.id}" class="after:absolute after:inset-0">
											{client.nom}
										</a>
									</Table.Cell>
									<Table.Cell>{client.email ?? '—'}</Table.Cell>
									<Table.Cell>{client.telephone ?? '—'}</Table.Cell>
								</Table.Row>
							{/snippet}
						</ContextMenu.Trigger>
						<ContextMenu.Content class="w-44">
							<ContextMenu.Item onSelect={() => goto(`/clients/${client.id}`)}>
								Ouvrir la fiche
							</ContextMenu.Item>
							<ContextMenu.Item
								onSelect={() => {
									rdvClientId = client.id;
									rdvDialogOpen = true;
								}}
							>
								Nouveau RDV
							</ContextMenu.Item>
						</ContextMenu.Content>
					</ContextMenu.Root>
				{/each}
			</Table.Body>
		</Table.Root>
	{/if}
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Nouveau client</Dialog.Title>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<Label for="client-nom">Nom</Label>
				<Input id="client-nom" bind:value={nom} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="client-email">Email</Label>
				<Input id="client-email" type="email" bind:value={email} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="client-telephone">Téléphone</Label>
				<Input id="client-telephone" bind:value={telephone} />
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Annuler</Button>
			<Button onclick={save} disabled={saving}>Enregistrer</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<RdvDialog
	open={rdvDialogOpen}
	presetClientId={rdvClientId}
	onClose={() => {
		rdvDialogOpen = false;
		rdvClientId = undefined;
	}}
	onSaved={() => {
		rdvDialogOpen = false;
		rdvClientId = undefined;
	}}
/>
