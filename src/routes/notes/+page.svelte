<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import SearchIcon from '@lucide/svelte/icons/search';
	import SquarePenIcon from '@lucide/svelte/icons/square-pen';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import { notesDelete, notesList, notesUpsert } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Note } from '$lib/types';
	import { formatNoteListDate } from '$lib/format';
	import { notePlainText, notePreview, noteTitle } from '$lib/notesHtml';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import { cn } from '$lib/utils.js';

	let notes = $state<Note[]>([]);
	let selectedId = $state<string | null>(null);
	let recherche = $state('');
	let initial = $state(true);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let pendingId: string | null = null;

	const selected = $derived(notes.find((n) => n.id === selectedId) ?? null);

	const filtered = $derived.by(() => {
		const q = recherche.trim().toLowerCase();
		if (!q) return notes;
		return notes.filter((n) => notePlainText(n.corps).toLowerCase().includes(q));
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
			toast.error(userMessage(e));
		} finally {
			initial = false;
		}
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
			toast.error(userMessage(e));
		}
	}

	function scheduleSave(note: Note) {
		pendingId = note.id;
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			void flushSave();
		}, 400);
	}

	function onHtml(html: string) {
		if (!selected) return;
		selected.corps = html;
		scheduleSave(selected);
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
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function removeNote(id: string) {
		if (selected?.id === id) {
			if (saveTimer) {
				clearTimeout(saveTimer);
				saveTimer = null;
			}
			pendingId = null;
		}
		try {
			await notesDelete(id);
			notes = notes.filter((n) => n.id !== id);
			if (selectedId === id) selectedId = notes[0]?.id ?? null;
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function removeSelected() {
		if (!selected) return;
		await removeNote(selected.id);
	}
</script>

<div class="absolute inset-0 flex min-h-0">
	<aside class="flex w-[17.5rem] shrink-0 flex-col border-r">
		<div class="flex items-center gap-1 px-2 py-2">
			<label class="relative min-w-0 flex-1">
				<span class="sr-only">Rechercher</span>
				<SearchIcon
					class="text-muted-foreground pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2"
				/>
				<input
					bind:value={recherche}
					placeholder="Rechercher"
					class="bg-muted placeholder:text-muted-foreground h-7 w-full rounded-md border-0 pr-2 pl-7 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
				/>
			</label>
			<Button
				variant="ghost"
				size="icon"
				class="size-7"
				onclick={createNote}
				aria-label="Nouvelle note"
			>
				<SquarePenIcon />
			</Button>
		</div>
		<div class="min-h-0 flex-1 overflow-y-auto px-1.5 pb-2">
			{#if initial}
				<div class="flex flex-col gap-1 p-1">
					<Skeleton class="h-12" />
					<Skeleton class="h-12" />
					<Skeleton class="h-12" />
				</div>
			{:else if filtered.length === 0}
				<p class="text-muted-foreground px-3 py-8 text-center text-sm">
					{notes.length === 0 ? 'Aucune note.' : 'Aucun résultat.'}
				</p>
			{:else}
				<ul>
					{#each filtered as note (note.id)}
						{@const isSelected = note.id === selectedId}
						<li>
							<ContextMenu.Root>
								<ContextMenu.Trigger>
									{#snippet child({ props })}
										<button
											{...props}
											type="button"
											class={cn(
												'w-full rounded-md px-2.5 py-2 text-left transition-colors',
												isSelected
													? 'bg-primary text-primary-foreground'
													: 'hover:bg-muted/70'
											)}
											onclick={() => selectNote(note.id)}
										>
											<p class="truncate text-sm font-medium">{noteTitle(note.corps)}</p>
											<p
												class={cn(
													'mt-0.5 truncate text-xs',
													isSelected ? 'text-primary-foreground/75' : 'text-muted-foreground'
												)}
											>
												<span class="font-mono">{formatNoteListDate(note.updated_at)}</span>
												{#if notePreview(note.corps)}
													<span> {notePreview(note.corps)}</span>
												{/if}
											</p>
										</button>
									{/snippet}
								</ContextMenu.Trigger>
								<ContextMenu.Content class="w-44">
									<ContextMenu.Item
										variant="destructive"
										onSelect={() => removeNote(note.id)}
									>
										Supprimer
									</ContextMenu.Item>
								</ContextMenu.Content>
							</ContextMenu.Root>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</aside>

	<section class="flex min-w-0 flex-1 flex-col">
		{#if selected}
			{#key selected.id}
				<div class="flex min-h-0 flex-1 flex-col">
					<NoteEditor noteId={selected.id} corps={selected.corps} onChange={onHtml}>
						{#snippet leading()}
							<Button
								variant="ghost"
								size="icon"
								class="size-7"
								onclick={removeSelected}
								aria-label="Supprimer la note"
							>
								<Trash2Icon />
							</Button>
						{/snippet}
					</NoteEditor>
				</div>
			{/key}
		{:else if !initial}
			<div class="text-muted-foreground flex flex-1 items-center justify-center text-sm">
				Sélectionnez une note ou créez-en une.
			</div>
		{/if}
	</section>
</div>
