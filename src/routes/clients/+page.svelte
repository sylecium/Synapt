<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { clientsDelete, clientsList, clientsUpsert, tarifsList } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Client, ClientStatut, Tarif } from '$lib/types';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import DateNaissanceField from '$lib/components/DateNaissanceField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import {
		CLIENT_FREQUENCE_LABELS,
		CLIENT_ORIENTATION_LABELS,
		CLIENT_STATUT_LABELS,
		frequenceLabel,
		orientationLabel,
		statutLabel
	} from '$lib/format';

	let clients = $state<Client[]>([]);
	let tarifs = $state<Tarif[]>([]);
	let recherche = $state('');
	let dialogOpen = $state(false);
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
	let showNaissance = $state(false);
	let showStatut = $state(false);
	let showTarif = $state(false);
	let showFrequence = $state(false);
	let showOrientation = $state(false);
	let showMemo = $state(false);
	let showUrgence = $state(false);
	let saving = $state(false);
	let initial = $state(true);
	let rdvDialogOpen = $state(false);
	let rdvClientId = $state<string | undefined>();
	let showTermines = $state(false);
	let deleteOpen = $state(false);
	let deleting = $state(false);
	let deleteTarget = $state<{ id: string; nom: string } | null>(null);

	const NONE = 'none';
	const extrasOpen = $derived(
		showNaissance ||
			showStatut ||
			showTarif ||
			showFrequence ||
			showOrientation ||
			showMemo ||
			showUrgence
	);
	const tarifsActifs = $derived(tarifs.filter((t) => t.actif));
	const tarifCourant = $derived(tarifs.find((t) => t.id === tarifId));
	const tarifSelectValue = $derived(tarifId === '' ? NONE : tarifId);
	const tarifLabel = $derived(
		tarifId === '' ? 'Sans tarif' : (tarifCourant?.nom ?? 'Sans tarif')
	);

	const filtered = $derived(
		clients.filter((c) => {
			if (!showTermines && c.statut === 'termine') return false;
			return c.nom.toLowerCase().includes(recherche.toLowerCase());
		})
	);

	onMount(load);

	async function load() {
		try {
			[clients, tarifs] = await Promise.all([clientsList(), tarifsList()]);
		} finally {
			initial = false;
		}
	}

	function resetForm() {
		nom = '';
		email = '';
		telephone = '';
		statut = 'en_cours';
		memo = '';
		tarifId = '';
		dateNaissance = '';
		urgenceNom = '';
		urgenceTelephone = '';
		orientation = '';
		frequence = '';
		showNaissance = false;
		showStatut = false;
		showTarif = false;
		showFrequence = false;
		showOrientation = false;
		showMemo = false;
		showUrgence = false;
	}

	function openCreate() {
		resetForm();
		dialogOpen = true;
	}

	function ignoreNestedOverlay(e: { target: EventTarget | null; preventDefault: () => void }) {
		const el = e.target instanceof Element ? e.target : null;
		if (
			el?.closest(
				'[data-slot="popover-content"], [data-slot="select-content"], [data-slot="dropdown-menu-content"]'
			)
		) {
			e.preventDefault();
		}
	}

	function statutBadge(s: Client['statut']): 'default' | 'secondary' | 'outline' {
		switch (s) {
			case 'en_cours':
				return 'default';
			case 'pause':
				return 'secondary';
			case 'termine':
				return 'outline';
			default: {
				const _n: never = s;
				return _n;
			}
		}
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
				telephone: telephone.trim() || null,
				statut,
				memo: memo.trim() || null,
				tarif_id: tarifId || null,
				date_naissance: dateNaissance || null,
				urgence_nom: urgenceNom.trim() || null,
				urgence_telephone: urgenceTelephone.trim() || null,
				orientation: orientation || null,
				frequence: frequence || null
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

	function askDelete(client: Client) {
		deleteTarget = { id: client.id, nom: client.nom };
		deleteOpen = true;
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		deleting = true;
		try {
			await clientsDelete(deleteTarget.id);
			toast.success('Client supprimé');
			deleteOpen = false;
			deleteTarget = null;
			await load();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			deleting = false;
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
		<div class="flex flex-wrap items-center gap-4">
			<Input placeholder="Rechercher par nom…" bind:value={recherche} class="max-w-sm" />
			<div class="flex items-center gap-2">
				<Switch id="show-termines" bind:checked={showTermines} />
				<Label for="show-termines">Afficher les suivis terminés</Label>
			</div>
		</div>

		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>Nom</Table.Head>
					<Table.Head>Statut</Table.Head>
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
									<Table.Cell>
										<Badge variant={statutBadge(client.statut)}>{statutLabel(client.statut)}</Badge>
									</Table.Cell>
									<Table.Cell>{client.email ?? '-'}</Table.Cell>
									<Table.Cell>{client.telephone ?? '-'}</Table.Cell>
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
							<ContextMenu.Item variant="destructive" onSelect={() => askDelete(client)}>
								Supprimer
							</ContextMenu.Item>
						</ContextMenu.Content>
					</ContextMenu.Root>
				{/each}
			</Table.Body>
		</Table.Root>
	{/if}
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content
		class="max-h-[min(40rem,85vh)] overflow-y-auto sm:max-w-md"
		onInteractOutside={ignoreNestedOverlay}
	>
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

			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button variant="outline" size="sm" class="w-fit" {...props}>
							<PlusIcon class="size-3.5" />
							Ajouter des champs
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content class="w-56" align="start">
					<DropdownMenu.Group>
						<DropdownMenu.GroupHeading>Champs optionnels</DropdownMenu.GroupHeading>
						<DropdownMenu.CheckboxItem bind:checked={showNaissance}>
							Date de naissance
						</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showStatut}>Statut</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showTarif}>
							Tarif habituel
						</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showFrequence}>
							Fréquence
						</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showOrientation}>
							Orientation
						</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showMemo}>Mémo</DropdownMenu.CheckboxItem>
						<DropdownMenu.CheckboxItem bind:checked={showUrgence}>
							Personne à prévenir
						</DropdownMenu.CheckboxItem>
					</DropdownMenu.Group>
				</DropdownMenu.Content>
			</DropdownMenu.Root>

			{#if extrasOpen}
				<div class="flex flex-col gap-4 border-t pt-4">
					{#if showNaissance}
						<div class="flex flex-col gap-2">
							<Label for="create-naissance">Date de naissance</Label>
							<DateNaissanceField id="create-naissance" bind:value={dateNaissance} />
						</div>
					{/if}
					{#if showStatut}
						<div class="flex flex-col gap-2">
							<Label>Statut</Label>
							<Select.Root
								type="single"
								value={statut}
								onValueChange={(v) => {
									if (v === 'en_cours' || v === 'pause' || v === 'termine') statut = v;
								}}
							>
								<Select.Trigger class="w-full">{CLIENT_STATUT_LABELS[statut]}</Select.Trigger>
								<Select.Content>
									{#each Object.entries(CLIENT_STATUT_LABELS) as [value, label] (value)}
										<Select.Item {value} {label}>{label}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>
					{/if}
					{#if showTarif}
						<div class="flex flex-col gap-2">
							<Label>Tarif habituel</Label>
							<Select.Root
								type="single"
								value={tarifSelectValue}
								onValueChange={(v) => (tarifId = !v || v === NONE ? '' : v)}
							>
								<Select.Trigger class="w-full">{tarifLabel}</Select.Trigger>
								<Select.Content>
									<Select.Item value={NONE} label="Sans tarif">Sans tarif</Select.Item>
									{#each tarifsActifs as tarif (tarif.id)}
										<Select.Item value={tarif.id} label={tarif.nom}>{tarif.nom}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>
					{/if}
					{#if showFrequence}
						<div class="flex flex-col gap-2">
							<Label>Fréquence</Label>
							<Select.Root
								type="single"
								value={frequence === '' ? NONE : frequence}
								onValueChange={(v) => (frequence = !v || v === NONE ? '' : v)}
							>
								<Select.Trigger class="w-full">{frequenceLabel(frequence)}</Select.Trigger>
								<Select.Content>
									<Select.Item value={NONE} label="Non renseignée">Non renseignée</Select.Item>
									{#each Object.entries(CLIENT_FREQUENCE_LABELS) as [value, label] (value)}
										<Select.Item {value} {label}>{label}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>
					{/if}
					{#if showOrientation}
						<div class="flex flex-col gap-2">
							<Label>Orientation</Label>
							<Select.Root
								type="single"
								value={orientation === '' ? NONE : orientation}
								onValueChange={(v) => (orientation = !v || v === NONE ? '' : v)}
							>
								<Select.Trigger class="w-full">{orientationLabel(orientation)}</Select.Trigger>
								<Select.Content>
									<Select.Item value={NONE} label="Non renseignée">Non renseignée</Select.Item>
									{#each Object.entries(CLIENT_ORIENTATION_LABELS) as [value, label] (value)}
										<Select.Item {value} {label}>{label}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>
					{/if}
					{#if showMemo}
						<div class="flex flex-col gap-2">
							<Label for="create-memo">Mémo</Label>
							<Input id="create-memo" bind:value={memo} maxlength={120} />
						</div>
					{/if}
					{#if showUrgence}
						<div class="flex flex-col gap-2">
							<Label for="create-urgence-nom">Personne à prévenir</Label>
							<Input id="create-urgence-nom" placeholder="Nom" bind:value={urgenceNom} />
							<Input
								id="create-urgence-tel"
								placeholder="Téléphone"
								bind:value={urgenceTelephone}
							/>
						</div>
					{/if}
				</div>
			{/if}
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

<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Supprimer {deleteTarget?.nom ?? 'ce client'} ?</Dialog.Title>
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
