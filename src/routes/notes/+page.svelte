<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { notesDelete, notesList, notesUpsert } from '$lib/api';
	import type { Note } from '$lib/types';
	import { formatDateTime } from '$lib/format';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';

	let notes = $state<Note[]>([]);
	let newCorps = $state('');
	let editingId = $state<string | null>(null);
	let loading = $state(true);

	onMount(load);

	async function load() {
		loading = true;
		try {
			notes = await notesList({ perso: true });
		} finally {
			loading = false;
		}
	}

	function noteTitle(corps: string): string {
		const line = corps.split('\n')[0];
		return line.length > 80 ? `${line.slice(0, 80)}…` : line;
	}

	function toggleEdit(id: string) {
		editingId = editingId === id ? null : id;
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
			if (editingId === id) editingId = null;
			toast.success('Note supprimée');
		} catch (e) {
			toast.error(String(e));
		}
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Notes personnelles" />

	<div class="flex max-w-2xl flex-col gap-2">
		<Textarea placeholder="Nouvelle note…" bind:value={newCorps} />
		<Button class="self-start" onclick={add}>Ajouter</Button>
	</div>

	{#if loading}
		<Skeleton class="h-32 max-w-2xl" />
	{:else if notes.length === 0}
		<EmptyState
			title="Première note personnelle"
			description="Une note pour vous, hors fiche client."
		/>
	{:else}
		<div class="max-w-2xl divide-y rounded-md border">
			{#each notes as note (note.id)}
				<div class="px-4 py-3">
					{#if editingId === note.id}
						<div class="flex flex-col gap-2">
							<Textarea bind:value={note.corps} />
							<div class="flex gap-2">
								<Button variant="outline" size="sm" onclick={() => save(note)}>
									Enregistrer
								</Button>
								<Button variant="destructive" size="sm" onclick={() => remove(note.id)}>
									Supprimer
								</Button>
							</div>
						</div>
					{:else}
						<button
							type="button"
							class="flex w-full items-center justify-between gap-4 text-left"
							onclick={() => toggleEdit(note.id)}
						>
							<span class="min-w-0 truncate text-sm">{noteTitle(note.corps)}</span>
							<span class="text-muted-foreground shrink-0 font-mono text-xs">
								{formatDateTime(note.updated_at)}
							</span>
						</button>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
