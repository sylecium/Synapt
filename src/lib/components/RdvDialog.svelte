<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { CalendarDate, type DateValue } from '@internationalized/date';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import { clientsList, rdvCreate, rdvGet, rdvUpdate, tarifsList } from '$lib/api';
	import type { Client, RdvCreateResult, Tarif } from '$lib/types';
	import { localDatetimeToUtcIso, utcIsoToLocalDatetime } from '$lib/format';
	import { noteIsEmpty } from '$lib/notesHtml';
	import { userMessage } from '$lib/errors';
	import ClientCombobox from '$lib/components/ClientCombobox.svelte';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Calendar from '$lib/components/ui/calendar/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Select from '$lib/components/ui/select/index.js';

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
	let calOpen = $state(false);
	let dureeMinutes = $state('60');
	let note = $state('');
	let saving = $state(false);
	let overlapError = $state('');
	let listsLoaded = $state(false);
	let formReady = $state(false);

	const NONE = 'none';
	const tarifsActifs = $derived(tarifs.filter((t) => t.actif));
	const tarifCourant = $derived(tarifs.find((t) => t.id === tarifId));
	const tarifSelectValue = $derived(tarifId === '' ? NONE : tarifId);
	const tarifLabel = $derived(
		tarifId === '' ? 'Sans tarif' : (tarifCourant?.nom ?? 'Sans tarif')
	);
	const dateValue = $derived.by((): DateValue | undefined => {
		const date = debutLocal.split('T')[0];
		if (!date) return undefined;
		const [y, m, d] = date.split('-').map(Number);
		if (!y || !m || !d) return undefined;
		return new CalendarDate(y, m, d);
	});
	const timeValue = $derived(debutLocal.split('T')[1] ?? '09:00');
	const dateLabel = $derived.by(() => {
		const date = debutLocal.split('T')[0];
		if (!date) return 'Date';
		const [y, m, d] = date.split('-');
		return `${d}/${m}/${y}`;
	});

	function pad(n: number): string {
		return String(n).padStart(2, '0');
	}

	function setDate(next: DateValue | undefined) {
		if (!next) return;
		const time = debutLocal.split('T')[1] ?? '09:00';
		debutLocal = `${next.year}-${pad(next.month)}-${pad(next.day)}T${time}`;
		calOpen = false;
	}

	function setTime(next: string) {
		const date = debutLocal.split('T')[0];
		if (!date || !next) return;
		debutLocal = `${date}T${next}`;
	}

	function ignoreNestedOverlay(e: { target: EventTarget | null; preventDefault: () => void }) {
		const el = e.target instanceof Element ? e.target : null;
		if (el?.closest('[data-slot="popover-content"], [data-slot="select-content"]')) {
			e.preventDefault();
		}
	}

	$effect(() => {
		if (open) {
			loadData();
		} else {
			calOpen = false;
		}
	});

	async function loadData() {
		overlapError = '';
		listsLoaded = false;
		formReady = false;
		[clients, tarifs] = await Promise.all([clientsList(), tarifsList()]);
		listsLoaded = true;

		if (rdvId) {
			const detail = await rdvGet(rdvId);
			const rdv = detail.rdv;
			clientId = rdv.client_id;
			tarifId = rdv.tarif_id ?? '';
			debutLocal = utcIsoToLocalDatetime(rdv.debut);
			dureeMinutes = String(rdv.duree_minutes);
			note = rdv.note ?? '';
			formReady = true;
			return;
		}

		clientId = presetClientId ?? clients[0]?.id ?? '';
		tarifId = tarifForNewRdv(clientId);
		const tarif = tarifs.find((t) => t.id === tarifId);
		dureeMinutes = String(tarif?.duree_minutes ?? 60);
		debutLocal = presetDebut ? utcIsoToLocalDatetime(presetDebut) : defaultDebutLocal();
		note = '';
		formReady = true;
	}

	function tarifForNewRdv(cid: string): string {
		const client = clients.find((c) => c.id === cid);
		const habitual = tarifs.find((t) => t.id === client?.tarif_id && t.actif);
		if (habitual) return habitual.id;
		return tarifs.filter((t) => t.actif)[0]?.id ?? '';
	}

	function defaultDebutLocal(): string {
		const d = new Date();
		d.setMinutes(Math.ceil(d.getMinutes() / 30) * 30, 0, 0);
		return utcIsoToLocalDatetime(d.toISOString());
	}

	function onTarifChange() {
		const tarif = tarifs.find((t) => t.id === tarifId);
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
				const result = await rdvUpdate({
					id: rdvId,
					client_id: clientId,
					tarif_id: tarifId || null,
					debut: localDatetimeToUtcIso(debutLocal),
					duree_minutes: Number.parseInt(dureeMinutes, 10),
					note: noteIsEmpty(note) ? null : note
				});
				for (const w of result.warnings) {
					toast.error(userMessage(w));
				}
				onSaved(result);
			} else {
				const result = await rdvCreate({
					client_id: clientId,
					tarif_id: tarifId || null,
					debut: localDatetimeToUtcIso(debutLocal),
					duree_minutes: Number.parseInt(dureeMinutes, 10),
					note: noteIsEmpty(note) ? null : note
				});
				for (const w of result.warnings) {
					toast.error(userMessage(w));
				}
				onSaved(result);
			}
			open = false;
		} catch (e) {
			const msg = userMessage(e);
			if (msg.includes('chevauche')) {
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
	<Dialog.Content class="sm:max-w-md" onInteractOutside={ignoreNestedOverlay}>
		<Dialog.Header>
			<Dialog.Title>{isEdit ? 'Modifier RDV' : 'Nouveau RDV'}</Dialog.Title>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<Label>Client</Label>
				<ClientCombobox
					{clients}
					value={clientId}
					loaded={listsLoaded}
					onValueChange={(id) => {
						clientId = id;
						if (!isEdit) {
							tarifId = tarifForNewRdv(id);
							onTarifChange();
						}
					}}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<Label>Tarif</Label>
				<Select.Root
					type="single"
					value={tarifSelectValue}
					onValueChange={(v) => {
						tarifId = !v || v === NONE ? '' : v;
						onTarifChange();
					}}
				>
					<Select.Trigger class="w-full">
						{tarifLabel}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value={NONE} label="Sans tarif">Sans tarif</Select.Item>
						{#if tarifCourant && !tarifCourant.actif}
							<Select.Item value={tarifCourant.id} label={tarifCourant.nom}>
								{tarifCourant.nom} (inactif)
							</Select.Item>
						{/if}
						{#each tarifsActifs as tarif (tarif.id)}
							<Select.Item value={tarif.id} label={tarif.nom}>{tarif.nom}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2">
				<Label>Début</Label>
				<div class="flex gap-2">
					<div class="min-w-0 flex-1">
						<Popover.Root bind:open={calOpen}>
							<Popover.Trigger>
								{#snippet child({ props })}
									<Button variant="outline" class="w-full justify-start" {...props}>
										<CalendarIcon class="size-3.5" />
										{dateLabel}
									</Button>
								{/snippet}
							</Popover.Trigger>
							<Popover.Content class="z-[60] w-auto p-0" align="start">
								<Calendar.Calendar
									type="single"
									value={dateValue}
									locale="fr-FR"
									onValueChange={setDate}
								/>
							</Popover.Content>
						</Popover.Root>
					</div>
					<Input
						id="rdv-debut-heure"
						type="time"
						class="w-[7.5rem]"
						value={timeValue}
						oninput={(e) => setTime(e.currentTarget.value)}
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<Label for="rdv-duree">Durée (minutes)</Label>
				<Input id="rdv-duree" type="number" min="1" bind:value={dureeMinutes} />
			</div>
			<div class="flex flex-col gap-2">
				<Label>Note</Label>
				{#if formReady}
					{#key `${rdvId ?? 'new'}:${open}`}
						<NoteEditor
							noteId={rdvId ?? 'new'}
							corps={note}
							variant="compact"
							onChange={(html) => (note = html)}
						/>
					{/key}
				{/if}
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
