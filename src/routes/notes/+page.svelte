<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { toast } from 'svelte-sonner';
	import SquarePenIcon from '@lucide/svelte/icons/square-pen';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import { notesDelete, notesList, notesUpsert } from '$lib/api';
	import type { Note } from '$lib/types';
	import { formatNoteListDate } from '$lib/format';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { cn } from '$lib/utils.js';

	let notes = $state<Note[]>([]);
	let selectedId = $state<string | null>(null);
	let recherche = $state('');
	let initial = $state(true);
	let editorEl = $state<HTMLTextAreaElement | null>(null);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let pendingId: string | null = null;

	const selected = $derived(notes.find((n) => n.id === selectedId) ?? null);

	const filtered = $derived.by(() => {
		const q = recherche.trim().toLowerCase();
		if (!q) return notes;
		return notes.filter((n) => n.corps.toLowerCase().includes(q));
	});

	onMount(load);
	onDestroy(() => {
		if (saveTimer) clearTimeout(saveTimer);
		void flushSave();
	});

	async function load() {
		try {
			notes = await notesList({ perso: true });
			if (!selectedId && notes[0]) selectedId = notes[0].id;
		} catch (e) {
			toast.error(String(e));
		} finally {
			initial = false;
		}
	}

	function noteTitle(corps: string): string {
		const line = corps.split('\n').find((l) => l.trim());
		return line?.trim() || 'Nouvelle note';
	}

	function notePreview(corps: string): string {
		const lines = corps.split('\n');
		const first = lines.findIndex((l) => l.trim());
		if (first < 0) return '';
		const rest = lines
			.slice(first + 1)
			.join(' ')
			.trim();
		return rest;
	}

	async function flushSave() {
		if (saveTimer) {
			clearTimeout(saveTimer);
			saveTimer = null;
		}
		const id = pendingId;
		if (!id) return;
		const note = notes.find((n) => n.id === id);
		pendingId = null;
		if (!note) return;
		try {
			const saved = await notesUpsert({
				id: note.id,
				client_id: null,
				corps: note.corps
			});
			note.updated_at = saved.updated_at;
			notes = [note, ...notes.filter((n) => n.id !== note.id)];
		} catch (e) {
			toast.error(String(e));
		}
	}

	function scheduleSave(note: Note) {
		pendingId = note.id;
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			void flushSave();
		}, 400);
	}

	async function selectNote(id: string) {
		if (id === selectedId) return;
		await flushSave();
		selectedId = id;
	}

	async function createNote() {
		await flushSave();
		try {
			const note = await notesUpsert({ client_id: null, corps: '' });
			notes = [note, ...notes];
			selectedId = note.id;
			await tick();
			editorEl?.focus();
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function removeSelected() {
		if (!selected) return;
		const id = selected.id;
		if (saveTimer) {
			clearTimeout(saveTimer);
			saveTimer = null;
		}
		pendingId = null;
		try {
			await notesDelete(id);
			notes = notes.filter((n) => n.id !== id);
			selectedId = notes[0]?.id ?? null;
		} catch (e) {
			toast.error(String(e));
		}
	}
</script>

<div class="absolute inset-0 flex min-h-0">
	<aside class="flex w-72 shrink-0 flex-col border-r">
		<div class="flex items-center gap-2 border-b px-3 py-2">
			<Input
				placeholder="Rechercher"
				bind:value={recherche}
				class="h-8"
				aria-label="Rechercher dans les notes"
			/>
			<Button variant="ghost" size="icon" onclick={createNote} aria-label="Nouvelle note">
				<SquarePenIcon />
			</Button>
		</div>
		<div class="min-h-0 flex-1 overflow-y-auto">
			{#if initial}
				<div class="flex flex-col gap-2 p-3">
					<Skeleton class="h-14" />
					<Skeleton class="h-14" />
					<Skeleton class="h-14" />
				</div>
			{:else if filtered.length === 0}
				<p class="text-muted-foreground px-3 py-6 text-sm">
					{notes.length === 0 ? 'Aucune note.' : 'Aucun résultat.'}
				</p>
			{:else}
				<ul>
					{#each filtered as note (note.id)}
						<li>
							<button
								type="button"
								class={cn(
									'hover:bg-muted/60 w-full border-b px-3 py-2.5 text-left',
									note.id === selectedId && 'bg-muted'
								)}
								onclick={() => selectNote(note.id)}
							>
								<p class="truncate text-sm font-medium">{noteTitle(note.corps)}</p>
								<p class="text-muted-foreground mt-0.5 truncate text-xs">
									<span class="font-mono">{formatNoteListDate(note.updated_at)}</span>
									{#if notePreview(note.corps)}
										<span> {notePreview(note.corps)}</span>
									{/if}
								</p>
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</aside>

	<section class="flex min-w-0 flex-1 flex-col">
		{#if selected}
			<div class="flex h-10 shrink-0 items-center justify-end border-b px-2">
				<Button
					variant="ghost"
					size="icon"
					onclick={removeSelected}
					aria-label="Supprimer la note"
				>
					<Trash2Icon />
				</Button>
			</div>
			<textarea
				bind:this={editorEl}
				class="placeholder:text-muted-foreground min-h-0 flex-1 resize-none bg-transparent px-6 py-4 text-sm leading-relaxed outline-none"
				placeholder="Nouvelle note"
				bind:value={selected.corps}
				oninput={() => selected && scheduleSave(selected)}
			></textarea>
		{:else if !initial}
			<div class="text-muted-foreground flex flex-1 items-center justify-center text-sm">
				Sélectionnez une note ou créez-en une.
			</div>
		{/if}
	</section>
</div>
