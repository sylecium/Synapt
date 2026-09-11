<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { notesDelete, notesList, notesUpsert } from '$lib/api';
	import type { Note } from '$lib/types';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';

	let notes = $state<Note[]>([]);
	let newCorps = $state('');

	onMount(load);

	async function load() {
		notes = await notesList({ perso: true });
	}

	async function save(note: Note) {
		try {
			await notesUpsert({ id: note.id, client_id: null, corps: note.corps });
			toast.success('Note enregistrée');
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function add() {
		if (!newCorps.trim()) return;
		try {
			const note = await notesUpsert({ client_id: null, corps: newCorps.trim() });
			notes = [note, ...notes];
			newCorps = '';
			toast.success('Note ajoutée');
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function remove(id: string) {
		try {
			await notesDelete(id);
			notes = notes.filter((n) => n.id !== id);
			toast.success('Note supprimée');
		} catch (e) {
			toast.error(String(e));
		}
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<h1 class="text-2xl font-semibold">Notes personnelles</h1>

	<div class="flex max-w-2xl flex-col gap-2">
		<Textarea placeholder="Nouvelle note…" bind:value={newCorps} />
		<Button class="self-start" onclick={add}>Ajouter</Button>
	</div>

	<div class="flex max-w-2xl flex-col gap-4">
		{#each notes as note (note.id)}
			<div class="flex flex-col gap-2 rounded-lg border p-4">
				<Textarea bind:value={note.corps} />
				<div class="flex gap-2">
					<Button variant="outline" size="sm" onclick={() => save(note)}>Enregistrer</Button>
					<Button variant="destructive" size="sm" onclick={() => remove(note.id)}>Supprimer</Button>
				</div>
			</div>
		{:else}
			<p class="text-muted-foreground text-sm">Aucune note</p>
		{/each}
	</div>
</div>
