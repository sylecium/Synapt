<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { clientsList, rdvCreate, rdvGet, rdvUpdate, tarifsList } from '$lib/api';
	import type { Client, RdvCreateResult, Tarif } from '$lib/types';
	import { localDatetimeToUtcIso, utcIsoToLocalDatetime } from '$lib/format';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	type Props = {
		open: boolean;
		rdvId?: string;
		presetDebut?: string;
		presetClientId?: string;
		onClose: () => void;
		onSaved: (r: RdvCreateResult) => void;
	};

	let {
		open = $bindable(),
		rdvId,
		presetDebut,
		presetClientId,
		onClose,
		onSaved
	}: Props = $props();

	const isEdit = $derived(!!rdvId);

	let clients = $state<Client[]>([]);
	let tarifs = $state<Tarif[]>([]);
	let clientId = $state('');
	let tarifId = $state('');
	let debutLocal = $state('');
	let dureeMinutes = $state('60');
	let note = $state('');
	let saving = $state(false);
	let overlapError = $state('');

	const tarifsActifs = $derived(tarifs.filter((t) => t.actif));

	$effect(() => {
		if (open) {
			loadData();
		}
	});

	async function loadData() {
		overlapError = '';
		[clients, tarifs] = await Promise.all([clientsList(), tarifsList()]);

		if (rdvId) {
			const detail = await rdvGet(rdvId);
			const rdv = detail.rdv;
			clientId = rdv.client_id;
			tarifId = rdv.tarif_id ?? '';
			debutLocal = utcIsoToLocalDatetime(rdv.debut);
			dureeMinutes = String(rdv.duree_minutes);
			note = rdv.note ?? '';
			return;
		}

		clientId = presetClientId ?? clients[0]?.id ?? '';
		const actifs = tarifs.filter((t) => t.actif);
		tarifId = actifs[0]?.id ?? '';
		dureeMinutes = String(actifs[0]?.duree_minutes ?? 60);
		debutLocal = presetDebut ? utcIsoToLocalDatetime(presetDebut) : defaultDebutLocal();
		note = '';
	}

	function defaultDebutLocal(): string {
		const d = new Date();
		d.setMinutes(Math.ceil(d.getMinutes() / 30) * 30, 0, 0);
		return utcIsoToLocalDatetime(d.toISOString());
	}

	function onTarifChange() {
		const tarif = tarifsActifs.find((t) => t.id === tarifId);
		if (tarif) dureeMinutes = String(tarif.duree_minutes);
	}

	function handleOpenChange(value: boolean) {
		open = value;
		if (!value) onClose();
	}

	async function submit() {
		if (!clientId) {
			toast.error('Sélectionnez un client');
			return;
		}
		overlapError = '';
		saving = true;
		try {
			if (isEdit && rdvId) {
				const rdv = await rdvUpdate({
					id: rdvId,
					client_id: clientId,
					tarif_id: tarifId || null,
					debut: localDatetimeToUtcIso(debutLocal),
					duree_minutes: Number.parseInt(dureeMinutes, 10),
					note: note.trim() || null
				});
				onSaved({ rdv, warnings: [] });
			} else {
				const result = await rdvCreate({
					client_id: clientId,
					tarif_id: tarifId || null,
					debut: localDatetimeToUtcIso(debutLocal),
					duree_minutes: Number.parseInt(dureeMinutes, 10),
					note: note.trim() || null
				});
				for (const w of result.warnings) {
					toast.error(w);
				}
				onSaved(result);
			}
			open = false;
		} catch (e) {
			const msg = String(e);
			if (msg.includes('chevauche') || msg.includes('horaire')) {
				overlapError = msg;
			} else {
				toast.error(msg);
			}
		} finally {
			saving = false;
		}
	}
</script>

<Dialog.Root open={open} onOpenChange={handleOpenChange}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{isEdit ? 'Modifier RDV' : 'Nouveau RDV'}</Dialog.Title>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<Label for="rdv-client">Client</Label>
				<select
					id="rdv-client"
					class="bg-input/20 dark:bg-input/30 border-input h-7 w-full rounded-md border px-2 text-sm"
					bind:value={clientId}
				>
					{#each clients as client (client.id)}
						<option value={client.id}>{client.nom}</option>
					{/each}
				</select>
			</div>
			<div class="flex flex-col gap-2">
				<Label for="rdv-tarif">Tarif</Label>
				<select
					id="rdv-tarif"
					class="bg-input/20 dark:bg-input/30 border-input h-7 w-full rounded-md border px-2 text-sm"
					bind:value={tarifId}
					onchange={onTarifChange}
				>
					<option value="">Sans tarif</option>
					{#each tarifsActifs as tarif (tarif.id)}
						<option value={tarif.id}>{tarif.nom}</option>
					{/each}
				</select>
			</div>
			<div class="flex flex-col gap-2">
				<Label for="rdv-debut">Début</Label>
				<Input id="rdv-debut" type="datetime-local" bind:value={debutLocal} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="rdv-duree">Durée (minutes)</Label>
				<Input id="rdv-duree" type="number" min="1" bind:value={dureeMinutes} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="rdv-note">Note</Label>
				<Textarea id="rdv-note" bind:value={note} />
			</div>
			{#if overlapError}
				<p class="text-destructive text-sm">{overlapError}</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => handleOpenChange(false)}>Annuler</Button>
			<Button onclick={submit} disabled={saving}>{isEdit ? 'Enregistrer' : 'Créer'}</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
