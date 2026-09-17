<script lang="ts">
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { openPath } from '@tauri-apps/plugin-opener';
	import {
		clientsList,
		honorairesCreate,
		honorairesOuvrir,
		honorairesRdvsDisponibles
	} from '$lib/api';
	import { userMessage } from '$lib/errors';
	import { formatDateTime, MOYEN_PAIEMENT_LABELS, moyenPaiementLabel } from '$lib/format';
	import type { Client, HonoraireDetail, MoyenPaiement, Rdv } from '$lib/types';
	import ClientCombobox from '$lib/components/ClientCombobox.svelte';
	import { ignoreNestedOverlay } from '$lib/utils';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';

	type Props = {
		open: boolean;
		clientId?: string;
		rdvIds?: string[];
		onClose: () => void;
		onSaved: (h: HonoraireDetail) => void;
	};

	let { open = $bindable(), clientId: fixedClientId, rdvIds: presetRdvIds, onClose, onSaved }: Props =
		$props();

	const clientLocked = $derived(fixedClientId !== undefined);

	let clients = $state<Client[]>([]);
	let selectedClientId = $state('');
	let rdvs = $state<Rdv[]>([]);
	let selectedRdvIds = $state<Set<string>>(new Set());
	let moyenPaiement = $state<MoyenPaiement>('especes');
	let loading = $state(false);
	let saving = $state(false);
	let listsLoaded = $state(false);

	$effect(() => {
		if (open) {
			loadForm();
		}
	});

	async function loadForm() {
		loading = true;
		listsLoaded = false;
		moyenPaiement = 'especes';
		try {
			if (!clientLocked) {
				clients = await clientsList();
				listsLoaded = true;
				selectedClientId = clients[0]?.id ?? '';
			} else {
				selectedClientId = fixedClientId ?? '';
				listsLoaded = true;
			}
			await loadRdvs();
			const preset = new Set(presetRdvIds ?? []);
			if (preset.size > 0) {
				selectedRdvIds = new Set(rdvs.filter((r) => preset.has(r.id)).map((r) => r.id));
			} else if (rdvs.length === 1) {
				selectedRdvIds = new Set([rdvs[0].id]);
			} else {
				selectedRdvIds = new Set();
			}
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			loading = false;
		}
	}

	async function loadRdvs() {
		if (!selectedClientId) {
			rdvs = [];
			return;
		}
		rdvs = await honorairesRdvsDisponibles(selectedClientId);
	}

	async function onClientChange(id: string) {
		selectedClientId = id;
		selectedRdvIds = new Set();
		loading = true;
		try {
			await loadRdvs();
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			loading = false;
		}
	}

	function toggleRdv(id: string) {
		const next = new Set(selectedRdvIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedRdvIds = next;
	}

	function handleOpenChange(value: boolean) {
		open = value;
		if (!value) onClose();
	}

	function toastCabinet(msg: string) {
		if (msg.toLowerCase().includes('nom du cabinet')) {
			toast.error(msg, {
				action: {
					label: 'Réglages',
					onClick: () => goto('/reglages')
				}
			});
		} else {
			toast.error(msg);
		}
	}

	async function submit() {
		if (selectedRdvIds.size === 0) {
			toast.error('Choisissez au moins une séance.');
			return;
		}
		saving = true;
		try {
			const detail = await honorairesCreate({
				rdv_ids: [...selectedRdvIds],
				moyen_paiement: moyenPaiement
			});
			const path = await honorairesOuvrir(detail.honoraire.id);
			await openPath(path);
			toast.success('Note d\'honoraires créée');
			onSaved(detail);
			open = false;
		} catch (e) {
			const msg = userMessage(e);
			if (msg.toLowerCase().includes('nom du cabinet')) {
				toastCabinet(msg);
			} else {
				toast.error(msg);
			}
		} finally {
			saving = false;
		}
	}

	function rdvLabel(rdv: Rdv): string {
		const parts = [formatDateTime(rdv.debut)];
		if (rdv.tarif_nom) parts.push(rdv.tarif_nom);
		return parts.join(' - ');
	}
</script>

<Dialog.Root open={open} onOpenChange={handleOpenChange}>
	<Dialog.Content class="sm:max-w-md" onInteractOutside={ignoreNestedOverlay}>
		<Dialog.Header>
			<Dialog.Title>Nouvelle note d'honoraires</Dialog.Title>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			{#if !clientLocked}
				<div class="flex flex-col gap-2">
					<Label>Client</Label>
					<ClientCombobox
						{clients}
						value={selectedClientId}
						loaded={listsLoaded}
						onValueChange={onClientChange}
					/>
				</div>
			{/if}

			<div class="flex flex-col gap-2">
				<Label>Séances</Label>
				{#if loading}
					<p class="text-muted-foreground text-sm">Chargement…</p>
				{:else if !selectedClientId}
					<p class="text-muted-foreground text-sm">Choisissez un client.</p>
				{:else if rdvs.length === 0}
					<p class="text-muted-foreground text-sm">Aucune séance disponible pour ce client.</p>
				{:else}
					<ul class="flex max-h-48 flex-col gap-2 overflow-y-auto rounded-md border p-2">
						{#each rdvs as rdv (rdv.id)}
							<li>
								<label class="flex cursor-pointer items-start gap-2 text-sm">
									<input
										type="checkbox"
										class="mt-0.5"
										checked={selectedRdvIds.has(rdv.id)}
										onchange={() => toggleRdv(rdv.id)}
									/>
									<span>{rdvLabel(rdv)}</span>
								</label>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			<div class="flex flex-col gap-2">
				<Label>Moyen de paiement</Label>
				<Select.Root
					type="single"
					value={moyenPaiement}
					onValueChange={(v) => {
						if (v) moyenPaiement = v as MoyenPaiement;
					}}
				>
					<Select.Trigger class="w-full">{moyenPaiementLabel(moyenPaiement)}</Select.Trigger>
					<Select.Content>
						{#each Object.entries(MOYEN_PAIEMENT_LABELS) as [value, label] (value)}
							<Select.Item {value} {label}>{label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => handleOpenChange(false)}>Annuler</Button>
			<Button onclick={submit} disabled={saving || loading || selectedRdvIds.size === 0}>
				Générer
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
