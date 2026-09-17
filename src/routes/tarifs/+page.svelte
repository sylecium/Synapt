<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { tarifsDelete, tarifsList, tarifsSetActif, tarifsUpsert } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Tarif } from '$lib/types';
	import { centimesToEuros, eurosToCentimes, formatTarifPrix } from '$lib/format';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	let tarifs = $state<Tarif[]>([]);
	let dialogOpen = $state(false);
	let editing = $state<Tarif | null>(null);
	let nom = $state('');
	let dureeMinutes = $state('60');
	let prixEuros = $state('50.00');
	let prixTtc = $state(true);
	let saving = $state(false);
	let initial = $state(true);
	let deleteOpen = $state(false);
	let deleteTarget = $state<{ id: string; nom: string } | null>(null);
	let deleting = $state(false);

	onMount(load);

	async function load() {
		try {
			tarifs = await tarifsList();
		} finally {
			initial = false;
		}
	}

	function openCreate() {
		editing = null;
		nom = '';
		dureeMinutes = '60';
		prixEuros = '50.00';
		prixTtc = true;
		dialogOpen = true;
	}

	function openEdit(tarif: Tarif) {
		editing = tarif;
		nom = tarif.nom;
		dureeMinutes = String(tarif.duree_minutes);
		prixEuros = centimesToEuros(tarif.prix_centimes);
		prixTtc = tarif.prix_ttc;
		dialogOpen = true;
	}

	async function save() {
		saving = true;
		try {
			await tarifsUpsert({
				id: editing?.id,
				nom,
				duree_minutes: Number.parseInt(dureeMinutes, 10),
				prix_centimes: eurosToCentimes(prixEuros),
				prix_ttc: prixTtc
			});
			dialogOpen = false;
			await load();
			toast.success(editing ? 'Tarif modifié' : 'Tarif créé');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			saving = false;
		}
	}

	async function setActif(id: string, actif: boolean) {
		try {
			await tarifsSetActif(id, actif);
			await load();
			toast.success(actif ? 'Tarif réactivé' : 'Tarif désactivé');
		} catch (e) {
			toast.error(userMessage(e));
		}
	}

	function askDelete(tarif: Tarif) {
		deleteTarget = { id: tarif.id, nom: tarif.nom };
		deleteOpen = true;
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		deleting = true;
		try {
			await tarifsDelete(deleteTarget.id);
			toast.success('Tarif supprimé');
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
	<PageHeader title="Tarifs">
		<Button onclick={openCreate}>Nouveau tarif</Button>
	</PageHeader>

	{#if initial}
		<Skeleton class="h-48" />
	{:else if tarifs.length === 0}
		<EmptyState
			title="Aucun tarif"
			description="Créer une prestation avec durée et prix."
			actionLabel="Nouveau tarif"
			onclick={openCreate}
		/>
	{:else}
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>Nom</Table.Head>
					<Table.Head>Durée</Table.Head>
					<Table.Head>Prix</Table.Head>
					<Table.Head>Statut</Table.Head>
					<Table.Head class="text-right">Actions</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each tarifs as tarif (tarif.id)}
					<ContextMenu.Root>
						<ContextMenu.Trigger>
							{#snippet child({ props })}
								<Table.Row
									{...props}
									class="hover:bg-muted/50 cursor-pointer"
									onclick={() => openEdit(tarif)}
								>
									<Table.Cell>{tarif.nom}</Table.Cell>
									<Table.Cell class="font-mono tabular-nums">{tarif.duree_minutes} min</Table.Cell>
									<Table.Cell class="font-mono tabular-nums">{formatTarifPrix(tarif)}</Table.Cell>
									<Table.Cell>
										{#if tarif.actif}
											<Badge>Actif</Badge>
										{:else}
											<Badge variant="secondary">Inactif</Badge>
										{/if}
									</Table.Cell>
									<Table.Cell class="text-right">
										<div class="flex justify-end gap-2">
											<Button
												variant="outline"
												size="sm"
												onclick={(e) => {
													e.stopPropagation();
													openEdit(tarif);
												}}
											>
												Modifier
											</Button>
											{#if tarif.actif}
												<Button
													variant="outline"
													size="sm"
													onclick={(e) => {
														e.stopPropagation();
														setActif(tarif.id, false);
													}}
												>
													Désactiver
												</Button>
											{:else}
												<Button
													variant="outline"
													size="sm"
													onclick={(e) => {
														e.stopPropagation();
														setActif(tarif.id, true);
													}}
												>
													Réactiver
												</Button>
											{/if}
											<Button
												variant="outline"
												size="sm"
												onclick={(e) => {
													e.stopPropagation();
													askDelete(tarif);
												}}
											>
												Supprimer
											</Button>
										</div>
									</Table.Cell>
								</Table.Row>
							{/snippet}
						</ContextMenu.Trigger>
						<ContextMenu.Content class="w-44">
							<ContextMenu.Item onSelect={() => openEdit(tarif)}>Modifier</ContextMenu.Item>
							{#if tarif.actif}
								<ContextMenu.Item onSelect={() => setActif(tarif.id, false)}>
									Désactiver
								</ContextMenu.Item>
							{:else}
								<ContextMenu.Item onSelect={() => setActif(tarif.id, true)}>
									Réactiver
								</ContextMenu.Item>
							{/if}
							<ContextMenu.Item variant="destructive" onSelect={() => askDelete(tarif)}>
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
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editing ? 'Modifier le tarif' : 'Nouveau tarif'}</Dialog.Title>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<Label for="tarif-nom">Nom</Label>
				<Input id="tarif-nom" bind:value={nom} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="tarif-duree">Durée (minutes)</Label>
				<Input id="tarif-duree" type="number" min="1" bind:value={dureeMinutes} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="tarif-prix">Prix (€)</Label>
				<Input id="tarif-prix" type="number" step="0.01" min="0" bind:value={prixEuros} />
				<label class="flex items-center gap-2 text-sm">
					<input id="tarif-prix-ttc" type="checkbox" bind:checked={prixTtc} />
					Prix TTC
				</label>
				<p class="text-muted-foreground text-xs">
					{#if prixTtc}
						Le montant saisi est TTC (TVA 20 % incluse).
					{:else}
						Le montant saisi est HT. TVA 20 % ajoutée à la facture et au paiement.
					{/if}
				</p>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Annuler</Button>
			<Button onclick={save} disabled={saving}>Enregistrer</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Supprimer {deleteTarget?.nom ?? 'ce tarif'} ?</Dialog.Title>
			<Dialog.Description>
				Les clients qui l'avaient comme tarif habituel n'en auront plus. Les rendez-vous prévus
				doivent d'abord être changés ou annulés.
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
