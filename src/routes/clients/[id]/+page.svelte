<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { openPath } from '@tauri-apps/plugin-opener';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import {
		clientsDelete,
		clientsGet,
		clientsUpsert,
		honorairesList,
		honorairesOuvrir,
		notesDelete,
		notesList,
		notesUpsert,
		rdvList,
		tarifsList
	} from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Client, ClientStatut, Honoraire, HonoraireDetail, HonoraireStatut, Note, Rdv, Tarif } from '$lib/types';
	import { formatCentimes, formatDateTime, formatNoteListDate } from '$lib/format';
	import { rdvBornes } from '$lib/rdvBornes';
	import { noteTitle } from '$lib/notesHtml';
	import { createDebouncedNoteSave } from '$lib/debouncedNoteSave';
	import ClientForm from '$lib/components/ClientForm.svelte';
	import HonoraireContextMenu from '$lib/components/HonoraireContextMenu.svelte';
	import HonoraireDialog from '$lib/components/HonoraireDialog.svelte';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import RdvContextMenu from '$lib/components/RdvContextMenu.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import RdvPanel from '$lib/components/RdvPanel.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Table from '$lib/components/ui/table/index.js';

	const clientId = $derived(page.params.id!);

	let client = $state<Client | null>(null);
	let nom = $state('');
	let email = $state('');
	let telephone = $state('');
	let statut = $state<ClientStatut>('en_cours');
	let memo = $state('');
	let tarifId = $state('');
	let dateNaissance = $state('');
	let urgenceNom = $state('');
	let urgenceTelephone = $state('');
	let orientation = $state('');
	let frequence = $state('');
	let adresse = $state('');
	let tarifs = $state<Tarif[]>([]);
	let notes = $state<Note[]>([]);
	let honoraires = $state<Honoraire[]>([]);
	let rdvs = $state<Rdv[]>([]);
	let saving = $state(false);
	let initial = $state(true);
	let rdvDialogOpen = $state(false);
	let editRdvId = $state<string | null>(null);
	let panelOpen = $state(false);
	let panelRdvId = $state<string | null>(null);
	let editingId = $state<string | null>(null);
	let deleteOpen = $state(false);
	let deleting = $state(false);
	let honoraireDialogOpen = $state(false);

	const bornes = $derived(rdvBornes(rdvs));

	const noteSave = createDebouncedNoteSave({
		getNote: (id) => notes.find((n) => n.id === id),
		save: (note) => notesUpsert({ id: note.id, client_id: clientId, corps: note.corps }),
		onSaved: (note) => {
			notes = [note, ...notes.filter((n) => n.id !== note.id)];
		},
		onError: (e) => toast.error(userMessage(e))
	});

	$effect(() => {
		const id = clientId;
		let cancelled = false;
		(async () => {
			await load(id, () => cancelled);
		})();
		return () => {
			cancelled = true;
			void noteSave.flush();
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
			statut = c.statut;
			memo = c.memo ?? '';
			tarifId = c.tarif_id ?? '';
			dateNaissance = c.date_naissance ?? '';
			urgenceNom = c.urgence_nom ?? '';
			urgenceTelephone = c.urgence_telephone ?? '';
			orientation = c.orientation ?? '';
			frequence = c.frequence ?? '';
			adresse = c.adresse ?? '';
			const [n, r, t, h] = await Promise.all([
				notesList({ client_id: id }),
				rdvList({ client_id: id }),
				tarifsList(),
				honorairesList(id)
			]);
			if (isCancelled()) return;
			notes = n;
			rdvs = r;
			tarifs = t;
			honoraires = h;
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
				telephone: telephone.trim() || null,
				statut,
				memo: memo.trim() || null,
				tarif_id: tarifId || null,
				date_naissance: dateNaissance || null,
				urgence_nom: urgenceNom.trim() || null,
				urgence_telephone: urgenceTelephone.trim() || null,
				orientation: orientation || null,
				frequence: frequence || null,
				adresse: adresse.trim() || null
			});
			toast.success('Client enregistré');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			saving = false;
		}
	}

	function onHtml(html: string) {
		const note = notes.find((n) => n.id === editingId);
		if (!note) return;
		note.corps = html;
		noteSave.schedule(note);
	}

	async function expandNote(id: string) {
		if (id === editingId) return;
		await noteSave.flush();
		editingId = id;
	}

	async function createNote() {
		await noteSave.flush();
		try {
			const note = await notesUpsert({ client_id: clientId, corps: '' });
			notes = [note, ...notes];
			editingId = note.id;
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	async function removeNote(id: string) {
		noteSave.cancel(id);
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

	function openEditRdv(rdv: Rdv) {
		editRdvId = rdv.id;
		rdvDialogOpen = true;
	}

	function onRdvSaved() {
		rdvDialogOpen = false;
		editRdvId = null;
		load(clientId);
	}

	function honoraireStatutLabel(statut: HonoraireStatut): string {
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

	function honoraireStatutBadge(statut: HonoraireStatut): 'default' | 'outline' {
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

	function onHonoraireSaved(_detail: HonoraireDetail) {
		honoraireDialogOpen = false;
		load(clientId);
	}

	async function confirmDelete() {
		deleting = true;
		try {
			await clientsDelete(clientId);
			toast.success('Client supprimé');
			await goto('/clients');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			deleting = false;
			deleteOpen = false;
		}
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title={client?.nom ?? 'Fiche client'}>
		<Button variant="outline" onclick={() => (deleteOpen = true)}>Supprimer</Button>
		<Button
			onclick={() => {
				editRdvId = null;
				rdvDialogOpen = true;
			}}>Nouveau RDV</Button
		>
	</PageHeader>
	{#if client && !initial}
		<p class="text-muted-foreground font-mono text-xs tabular-nums">
			Prochain : {bornes.prochain ? formatDateTime(bornes.prochain.debut) : '-'}
			<span class="text-border mx-2">|</span>
			Dernier : {bornes.dernier ? formatDateTime(bornes.dernier.debut) : '-'}
		</p>
	{/if}

	{#if initial}
		<Skeleton class="h-48 max-w-lg" />
		<Skeleton class="h-32" />
		<Skeleton class="h-32" />
	{:else if client}
		<section class="flex max-w-xl flex-col gap-6">
			<ClientForm
				variant="edit"
				{tarifs}
				bind:nom
				bind:email
				bind:telephone
				bind:statut
				bind:memo
				bind:tarifId
				bind:dateNaissance
				bind:urgenceNom
				bind:urgenceTelephone
				bind:orientation
				bind:frequence
				bind:adresse
			/>
			<Button onclick={saveClient} disabled={saving}>Enregistrer</Button>
		</section>

		<section class="flex max-w-2xl flex-col gap-3">
			<div class="flex items-center justify-between gap-2">
				<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">Notes</h2>
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
			<div class="flex items-center justify-between gap-2">
				<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
					Notes d'honoraires
				</h2>
				<Button variant="outline" size="sm" onclick={() => (honoraireDialogOpen = true)}>
					Regrouper
				</Button>
			</div>
			{#if honoraires.length === 0}
				<p class="text-muted-foreground text-sm">Aucune note d'honoraires pour ce client.</p>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Numéro</Table.Head>
							<Table.Head>Date</Table.Head>
							<Table.Head>Total</Table.Head>
							<Table.Head>Statut</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each honoraires as h (h.id)}
							<HonoraireContextMenu honoraire={h} onUpdated={() => load(clientId)}>
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
										<Table.Cell class="font-mono tabular-nums">
											{formatCentimes(h.total_centimes)}
										</Table.Cell>
										<Table.Cell>
											<Badge variant={honoraireStatutBadge(h.statut)}>
												{honoraireStatutLabel(h.statut)}
											</Badge>
										</Table.Cell>
									</Table.Row>
								{/snippet}
							</HonoraireContextMenu>
						{/each}
					</Table.Body>
				</Table.Root>
			{/if}
		</section>

		<section class="flex flex-col gap-4">
			<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
				Historique RDV
			</h2>
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
						<RdvContextMenu
							rdv={rdv}
							onOpen={() => openPanel(rdv)}
							onEdit={() => openEditRdv(rdv)}
							onUpdated={() => load(clientId)}
						>
							{#snippet children(props)}
								<Table.Row class="cursor-pointer" {...props} onclick={() => openPanel(rdv)}>
									<Table.Cell>{formatDateTime(rdv.debut)}</Table.Cell>
									<Table.Cell>{rdv.tarif_nom || '-'}</Table.Cell>
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
	rdvId={editRdvId ?? undefined}
	presetClientId={clientId}
	onClose={() => {
		rdvDialogOpen = false;
		editRdvId = null;
	}}
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

<HonoraireDialog
	bind:open={honoraireDialogOpen}
	clientId={clientId}
	onClose={() => (honoraireDialogOpen = false)}
	onSaved={onHonoraireSaved}
/>

<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Supprimer {client?.nom ?? 'ce client'} ?</Dialog.Title>
			<Dialog.Description>
				Les notes et les rendez-vous annulés seront aussi supprimés.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Annuler</Button>
			<Button variant="destructive" onclick={confirmDelete} disabled={deleting}>
				Supprimer
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
