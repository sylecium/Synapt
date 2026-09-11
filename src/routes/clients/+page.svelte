<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { clientsList, clientsUpsert } from '$lib/api';
	import type { Client } from '$lib/types';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	let clients = $state<Client[]>([]);
	let recherche = $state('');
	let dialogOpen = $state(false);
	let nom = $state('');
	let email = $state('');
	let telephone = $state('');
	let saving = $state(false);

	const filtered = $derived(
		clients.filter((c) => c.nom.toLowerCase().includes(recherche.toLowerCase()))
	);

	onMount(load);

	async function load() {
		clients = await clientsList();
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
			toast.error(String(e));
		} finally {
			saving = false;
		}
	}
</script>

<div class="flex flex-col gap-4 p-6">
	<div class="flex items-center justify-between gap-4">
		<h1 class="text-2xl font-semibold">Clients</h1>
		<Button onclick={openCreate}>Nouveau client</Button>
	</div>

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
				<Table.Row>
					<Table.Cell>
						<a href="/clients/{client.id}" class="font-medium hover:underline">{client.nom}</a>
					</Table.Cell>
					<Table.Cell>{client.email ?? '—'}</Table.Cell>
					<Table.Cell>{client.telephone ?? '—'}</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
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
