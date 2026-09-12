<script lang="ts">
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import {
		clientsGet,
		clientsUpsert,
		notesDelete,
		notesList,
		notesUpsert,
		rdvList
	} from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Client, Note, Rdv } from '$lib/types';
	import { formatDateTime, formatNoteListDate } from '$lib/format';
	import { noteTitle } from '$lib/notesHtml';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import RdvContextMenu from '$lib/components/RdvContextMenu.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import RdvPanel from '$lib/components/RdvPanel.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as Table from '$lib/components/ui/table/index.js';

	const clientId = $derived(page.params.id!);

	let client = $state<Client | null>(null);
	let nom = $state('');
	let email = $state('');
	let telephone = $state('');
	let notes = $state<Note[]>([]);
	let rdvs = $state<Rdv[]>([]);
	let saving = $state(false);
	let initial = $state(true);
	let rdvDialogOpen = $state(false);
	let panelOpen = $state(false);
	let panelRdvId = $state<string | null>(null);
	let editingId = $state<string | null>(null);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let pendingId: string | null = null;

	$effect(() => {
		const id = clientId;
		let cancelled = false;
		(async () => {
			await load(id, () => cancelled);
		})();
		return () => {
			cancelled = true;
			if (saveTimer) clearTimeout(saveTimer);
			void flushSave();
		};
	});

	async function load(id: string, isCancelled: () => boolean = () => false) {
		try {
			const c = await clientsGet(id);
			if (isCancelled()) return;
			client = c;
			nom = c.nom;
			email = c.email ?? '';
			telephone = c.telephone ?? '';
			const [n, r] = await Promise.all([
				notesList({ client_id: id }),
				rdvList({ client_id: id })
			]);
			if (isCancelled()) return;
			notes = n;
			rdvs = r;
			if (editingId && !n.some((note) => note.id === editingId)) {
				editingId = n[0]?.id ?? null;
			}
		} catch (e) {
			if (!isCancelled()) toast.error(userMessage(e));
		} finally {
			if (!isCancelled()) initial = false;
		}
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
			toast.error(userMessage(e));
		} finally {
			saving = false;
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
				client_id: clientId,
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
		const note = notes.find((n) => n.id === editingId);
		if (!note) return;
		note.corps = html;
		scheduleSave(note);
	}

	async function expandNote(id: string) {
		if (id === editingId) return;
		await flushSave();
		editingId = id;
	}

	async function createNote() {
		await flushSave();
		try {
			const note = await notesUpsert({ client_id: clientId, corps: '' });
			notes = [note, ...notes];
			editingId = note.id;
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function removeNote(id: string) {
		if (saveTimer && pendingId === id) {
			clearTimeout(saveTimer);
			saveTimer = null;
			pendingId = null;
		}
		try {
			await notesDelete(id);
			notes = notes.filter((n) => n.id !== id);
			if (editingId === id) editingId = notes[0]?.id ?? null;
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	function openPanel(rdv: Rdv) {
		panelRdvId = rdv.id;
		panelOpen = true;
	}

	function onRdvSaved() {
		rdvDialogOpen = false;
		load(clientId);
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title={client?.nom ?? 'Fiche client'}>
		<Button onclick={() => (rdvDialogOpen = true)}>Nouveau RDV</Button>
	</PageHeader>

	{#if initial}
		<Skeleton class="h-48 max-w-lg" />
		<Skeleton class="h-32" />
		<Skeleton class="h-32" />
	{:else if client}
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

		<section class="flex max-w-2xl flex-col gap-3">
			<div class="flex items-center justify-between gap-2">
				<h2 class="text-sm font-medium">Notes</h2>
				<Button variant="outline" size="sm" onclick={createNote}>Nouvelle note</Button>
			</div>
			{#if notes.length === 0}
				<p class="text-muted-foreground text-sm">Aucune note client.</p>
			{:else}
				<ul class="divide-y rounded-lg border">
					{#each notes as note (note.id)}
						<li>
							{#if editingId === note.id}
								<div class="p-2">
									<div class="flex items-center justify-between px-2 pb-1">
										<p class="text-muted-foreground font-mono text-xs">
											{formatNoteListDate(note.updated_at)}
										</p>
									</div>
									{#key note.id}
										<NoteEditor
											noteId={note.id}
											corps={note.corps}
											variant="compact"
											onChange={onHtml}
										>
											{#snippet leading()}
												<Button
													variant="ghost"
													size="icon"
													class="size-7"
													onclick={() => removeNote(note.id)}
													aria-label="Supprimer la note"
												>
													<Trash2Icon />
												</Button>
											{/snippet}
										</NoteEditor>
									{/key}
								</div>
							{:else}
								<ContextMenu.Root>
									<ContextMenu.Trigger>
										{#snippet child({ props })}
											<button
												{...props}
												type="button"
												class="w-full px-3 py-2.5 text-left hover:bg-muted/50"
												onclick={() => expandNote(note.id)}
											>
												<p class="truncate text-sm font-medium">{noteTitle(note.corps)}</p>
												<p class="text-muted-foreground mt-0.5 font-mono text-xs">
													{formatNoteListDate(note.updated_at)}
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
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
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
						<RdvContextMenu rdv={rdv} onOpen={() => openPanel(rdv)} onUpdated={() => load(clientId)}>
							{#snippet children(props)}
								<Table.Row class="cursor-pointer" {...props} onclick={() => openPanel(rdv)}>
									<Table.Cell>{formatDateTime(rdv.debut)}</Table.Cell>
									<Table.Cell>{rdv.tarif_nom || '—'}</Table.Cell>
									<Table.Cell>{rdv.duree_minutes} min</Table.Cell>
								</Table.Row>
							{/snippet}
						</RdvContextMenu>
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

<RdvPanel
	bind:open={panelOpen}
	rdvId={panelRdvId}
	onClose={() => {
		panelRdvId = null;
	}}
	onUpdated={() => load(clientId)}
/>
