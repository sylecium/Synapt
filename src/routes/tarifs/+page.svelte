<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { tarifsList, tarifsSetActif, tarifsUpsert } from '$lib/api';
	import type { Tarif } from '$lib/types';
	import { centimesToEuros, eurosToCentimes, formatCentimes } from '$lib/format';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	let tarifs = $state<Tarif[]>([]);
	let dialogOpen = $state(false);
	let editing = $state<Tarif | null>(null);
	let nom = $state('');
	let dureeMinutes = $state('60');
	let prixEuros = $state('50.00');
	let saving = $state(false);

	onMount(load);

	async function load() {
		tarifs = await tarifsList();
	}

	function openCreate() {
		editing = null;
		nom = '';
		dureeMinutes = '60';
		prixEuros = '50.00';
		dialogOpen = true;
	}

	function openEdit(tarif: Tarif) {
		editing = tarif;
		nom = tarif.nom;
		dureeMinutes = String(tarif.duree_minutes);
		prixEuros = centimesToEuros(tarif.prix_centimes);
		dialogOpen = true;
	}

	async function save() {
		saving = true;
		try {
			await tarifsUpsert({
				id: editing?.id,
				nom,
				duree_minutes: Number.parseInt(dureeMinutes, 10),
				prix_centimes: eurosToCentimes(prixEuros)
			});
			dialogOpen = false;
			await load();
			toast.success(editing ? 'Tarif modifié' : 'Tarif créé');
		} catch (e) {
			toast.error(String(e));
		} finally {
			saving = false;
		}
	}

	async function desactiver(id: string) {
		try {
			await tarifsSetActif(id, false);
			await load();
			toast.success('Tarif désactivé');
		} catch (e) {
			toast.error(String(e));
		}
	}
</script>

<div class="flex flex-col gap-4 p-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Tarifs</h1>
		<Button onclick={openCreate}>Nouveau tarif</Button>
	</div>

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
				<Table.Row>
					<Table.Cell>{tarif.nom}</Table.Cell>
					<Table.Cell>{tarif.duree_minutes} min</Table.Cell>
					<Table.Cell>{formatCentimes(tarif.prix_centimes)}</Table.Cell>
					<Table.Cell>
						{#if tarif.actif}
							<Badge>Actif</Badge>
						{:else}
							<Badge variant="secondary">Inactif</Badge>
						{/if}
					</Table.Cell>
					<Table.Cell class="text-right">
						<div class="flex justify-end gap-2">
							<Button variant="outline" size="sm" onclick={() => openEdit(tarif)}>Modifier</Button>
							{#if tarif.actif}
								<Button variant="outline" size="sm" onclick={() => desactiver(tarif.id)}>
									Désactiver
								</Button>
							{/if}
						</div>
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
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
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Annuler</Button>
			<Button onclick={save} disabled={saving}>Enregistrer</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
