<script lang="ts">
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';
	import { clientsGet, clientsUpsert, notesList, notesUpsert, rdvList } from '$lib/api';
	import type { Client, Note, Rdv } from '$lib/types';
	import { formatDateTime } from '$lib/format';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as Table from '$lib/components/ui/table/index.js';

	const clientId = $derived(page.params.id!);

	let client = $state<Client | null>(null);
	let nom = $state('');
	let email = $state('');
	let telephone = $state('');
	let notes = $state<Note[]>([]);
	let rdvs = $state<Rdv[]>([]);
	let saving = $state(false);
	let rdvDialogOpen = $state(false);
	let newNoteCorps = $state('');

	$effect(() => {
		load(clientId);
	});

	async function load(id: string) {
		client = await clientsGet(id);
		nom = client.nom;
		email = client.email ?? '';
		telephone = client.telephone ?? '';
		newNoteCorps = '';
		notes = await notesList({ client_id: id });
		rdvs = await rdvList({ client_id: id });
	}

	async function saveClient() {
		if (!client) return;
		saving = true;
		try {
			client = await clientsUpsert({
				id: client.id,
				nom: nom.trim(),
				email: email.trim() || null,
				telephone: telephone.trim() || null
			});
			toast.success('Client enregistré');
		} catch (e) {
			toast.error(String(e));
		} finally {
			saving = false;
		}
	}

	async function saveNote(note: Note) {
		try {
			await notesUpsert({ id: note.id, client_id: clientId, corps: note.corps });
			toast.success('Note enregistrée');
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function addNote() {
		if (!newNoteCorps.trim()) return;
		try {
			const note = await notesUpsert({ client_id: clientId, corps: newNoteCorps.trim() });
			notes = [...notes, note];
			newNoteCorps = '';
			toast.success('Note ajoutée');
		} catch (e) {
			toast.error(String(e));
		}
	}

	function onRdvSaved() {
		rdvDialogOpen = false;
		load(clientId);
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Fiche client</h1>
		<Button onclick={() => (rdvDialogOpen = true)}>Nouveau RDV</Button>
	</div>

	{#if client}
		<section class="flex max-w-lg flex-col gap-4">
			<div class="flex flex-col gap-2">
				<Label for="nom">Nom</Label>
				<Input id="nom" bind:value={nom} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="email">Email</Label>
				<Input id="email" type="email" bind:value={email} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="telephone">Téléphone</Label>
				<Input id="telephone" bind:value={telephone} />
			</div>
			<Button onclick={saveClient} disabled={saving}>Enregistrer</Button>
		</section>

		<section class="flex flex-col gap-4">
			<h2 class="text-sm font-medium">Notes</h2>
			{#each notes as note (note.id)}
				<div class="flex flex-col gap-2">
					<Textarea bind:value={note.corps} />
					<Button variant="outline" size="sm" class="self-start" onclick={() => saveNote(note)}>
						Enregistrer
					</Button>
				</div>
			{/each}
			<div class="flex flex-col gap-2">
				<Textarea placeholder="Nouvelle note…" bind:value={newNoteCorps} />
				<Button variant="outline" size="sm" class="self-start" onclick={addNote}>Ajouter</Button>
			</div>
		</section>

		<section class="flex flex-col gap-4">
			<h2 class="text-sm font-medium">Historique RDV</h2>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Date</Table.Head>
						<Table.Head>Tarif</Table.Head>
						<Table.Head>Durée</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each rdvs as rdv (rdv.id)}
						<Table.Row>
							<Table.Cell>{formatDateTime(rdv.debut)}</Table.Cell>
							<Table.Cell>{rdv.tarif_nom || '—'}</Table.Cell>
							<Table.Cell>{rdv.duree_minutes} min</Table.Cell>
						</Table.Row>
					{:else}
						<Table.Row>
							<Table.Cell colspan={3} class="text-muted-foreground">Aucun RDV</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</section>
	{/if}
</div>

<RdvDialog
	open={rdvDialogOpen}
	presetClientId={clientId}
	onClose={() => (rdvDialogOpen = false)}
	onSaved={onRdvSaved}
/>
