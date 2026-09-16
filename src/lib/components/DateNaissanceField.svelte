<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { parseDdMmYyyy, ymdToDdMmYyyy } from '$lib/dateNaissance';

	type Props = {
		id?: string;
		value?: string;
	};

	let { id = 'date-naissance', value = $bindable('') }: Props = $props();

	let typed = $state('');
	let fromParent = $state('');

	$effect(() => {
		if (value === fromParent) return;
		fromParent = value;
		typed = value ? ymdToDdMmYyyy(value) : '';
	});

	function applyYmd(next: string) {
		value = next;
		fromParent = next;
		typed = next ? ymdToDdMmYyyy(next) : '';
	}

	function onTyped(raw: string) {
		typed = raw;
		if (!raw.trim()) {
			applyYmd('');
			return;
		}
		const ymd = parseDdMmYyyy(raw);
		if (ymd) applyYmd(ymd);
	}
</script>

<div class="relative">
	<Input
		{id}
		class="font-mono pr-10"
		inputmode="numeric"
		placeholder="jj/mm/aaaa"
		value={typed}
		oninput={(e) => onTyped(e.currentTarget.value)}
		aria-label="Date de naissance au format jj/mm/aaaa"
	/>
	{#if value}
		<div class="absolute inset-y-0 right-0 flex items-center pr-0.5">
			<Button
				variant="ghost"
				size="icon-sm"
				onclick={() => applyYmd('')}
				aria-label="Effacer la date"
			>
				<XIcon />
			</Button>
		</div>
	{/if}
</div>
