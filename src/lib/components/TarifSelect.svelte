<script lang="ts">
	import type { Tarif } from '$lib/types';
	import * as Select from '$lib/components/ui/select/index.js';

	const NONE = 'none';

	type Props = {
		tarifs: Tarif[];
		value: string;
		onValueChange: (id: string) => void;
	};

	let { tarifs, value, onValueChange }: Props = $props();

	const tarifsActifs = $derived(tarifs.filter((t) => t.actif));
	const tarifCourant = $derived(tarifs.find((t) => t.id === value));
	const selectValue = $derived(value === '' ? NONE : value);
	const label = $derived(value === '' ? 'Sans tarif' : (tarifCourant?.nom ?? 'Sans tarif'));
</script>

<Select.Root
	type="single"
	value={selectValue}
	onValueChange={(v) => onValueChange(!v || v === NONE ? '' : v)}
>
	<Select.Trigger class="w-full">{label}</Select.Trigger>
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
