<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import type { ClientStatut, Tarif } from '$lib/types';
	import DateNaissanceField from '$lib/components/DateNaissanceField.svelte';
	import TarifSelect from '$lib/components/TarifSelect.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import {
		CLIENT_FREQUENCE_LABELS,
		CLIENT_ORIENTATION_LABELS,
		CLIENT_STATUT_LABELS,
		frequenceLabel,
		orientationLabel
	} from '$lib/format';

	const NONE = 'none';

	type Props = {
		variant: 'create' | 'edit';
		tarifs: Tarif[];
		open?: boolean;
		nom: string;
		email: string;
		telephone: string;
		statut: ClientStatut;
		memo: string;
		tarifId: string;
		dateNaissance: string;
		urgenceNom: string;
		urgenceTelephone: string;
		orientation: string;
		frequence: string;
	};

	let {
		variant,
		tarifs,
		open = false,
		nom = $bindable(),
		email = $bindable(),
		telephone = $bindable(),
		statut = $bindable(),
		memo = $bindable(),
		tarifId = $bindable(),
		dateNaissance = $bindable(),
		urgenceNom = $bindable(),
		urgenceTelephone = $bindable(),
		orientation = $bindable(),
		frequence = $bindable()
	}: Props = $props();

	let showNaissance = $state(false);
	let showStatut = $state(false);
	let showTarif = $state(false);
	let showFrequence = $state(false);
	let showOrientation = $state(false);
	let showMemo = $state(false);
	let showUrgence = $state(false);

	const extrasOpen = $derived(
		showNaissance ||
			showStatut ||
			showTarif ||
			showFrequence ||
			showOrientation ||
			showMemo ||
			showUrgence
	);

	$effect(() => {
		if (variant !== 'create' || open) return;
		showNaissance = false;
		showStatut = false;
		showTarif = false;
		showFrequence = false;
		showOrientation = false;
		showMemo = false;
		showUrgence = false;
	});

	function onStatutChange(v: string | undefined) {
		if (v === 'en_cours' || v === 'pause' || v === 'termine') statut = v;
	}
</script>

{#if variant === 'edit'}
	<div class="flex flex-col gap-6">
	<div class="flex flex-col gap-3">
		<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">Identité</h2>
		<div class="flex flex-col gap-2">
			<Label for="nom">Nom</Label>
			<Input id="nom" bind:value={nom} />
		</div>
		<div class="grid gap-3 sm:grid-cols-2">
			<div class="flex flex-col gap-2">
				<Label for="email">Email</Label>
				<Input id="email" type="email" bind:value={email} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="telephone">Téléphone</Label>
				<Input id="telephone" bind:value={telephone} />
			</div>
		</div>
		<div class="flex flex-col gap-2">
			<Label for="date-naissance">Date de naissance</Label>
			<DateNaissanceField id="date-naissance" bind:value={dateNaissance} />
		</div>
	</div>

	<div class="flex flex-col gap-3">
		<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">Suivi</h2>
		<div class="grid gap-3 sm:grid-cols-2">
			<div class="flex flex-col gap-2">
				<Label>Statut</Label>
				<Select.Root type="single" value={statut} onValueChange={onStatutChange}>
					<Select.Trigger class="w-full">{CLIENT_STATUT_LABELS[statut]}</Select.Trigger>
					<Select.Content>
						{#each Object.entries(CLIENT_STATUT_LABELS) as [value, label] (value)}
							<Select.Item {value} {label}>{label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2">
				<Label>Tarif habituel</Label>
				<TarifSelect {tarifs} value={tarifId} onValueChange={(id) => (tarifId = id)} />
			</div>
			<div class="flex flex-col gap-2">
				<Label>Fréquence</Label>
				<Select.Root
					type="single"
					value={frequence === '' ? NONE : frequence}
					onValueChange={(v) => (frequence = !v || v === NONE ? '' : v)}
				>
					<Select.Trigger class="w-full">
						{frequenceLabel(frequence)}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value={NONE} label="Non renseignée">Non renseignée</Select.Item>
						{#each Object.entries(CLIENT_FREQUENCE_LABELS) as [value, label] (value)}
							<Select.Item {value} {label}>{label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2">
				<Label>Orientation</Label>
				<Select.Root
					type="single"
					value={orientation === '' ? NONE : orientation}
					onValueChange={(v) => (orientation = !v || v === NONE ? '' : v)}
				>
					<Select.Trigger class="w-full">
						{orientationLabel(orientation)}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value={NONE} label="Non renseignée">Non renseignée</Select.Item>
						{#each Object.entries(CLIENT_ORIENTATION_LABELS) as [value, label] (value)}
							<Select.Item {value} {label}>{label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2 sm:col-span-2">
				<Label for="memo">Mémo</Label>
				<Input id="memo" bind:value={memo} maxlength={120} />
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-3">
		<h2 class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
			Personne à prévenir
		</h2>
		<div class="grid gap-3 sm:grid-cols-2">
			<div class="flex flex-col gap-2">
				<Label for="urgence-nom">Nom</Label>
				<Input id="urgence-nom" bind:value={urgenceNom} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="urgence-tel">Téléphone</Label>
				<Input id="urgence-tel" bind:value={urgenceTelephone} />
			</div>
		</div>
	</div>
	</div>
{:else}
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
					<Select.Root type="single" value={statut} onValueChange={onStatutChange}>
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
					<TarifSelect {tarifs} value={tarifId} onValueChange={(id) => (tarifId = id)} />
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
{/if}
