<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import XIcon from '@lucide/svelte/icons/x';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import {
		monthAllowsDay,
		parseDdMmYyyy,
		todayCalendar,
		yearAllowsDate,
		ymdToDateValue,
		ymdToDdMmYyyy
	} from '$lib/dateNaissance';

	type Step = 'day' | 'month' | 'year';

	type Props = {
		id?: string;
		value?: string;
	};

	let { id = 'date-naissance', value = $bindable('') }: Props = $props();

	const MONTHS = [
		'janvier',
		'février',
		'mars',
		'avril',
		'mai',
		'juin',
		'juillet',
		'août',
		'septembre',
		'octobre',
		'novembre',
		'décembre'
	] as const;

	const maxYear = todayCalendar().year;
	const allYears = Array.from({ length: maxYear - 1899 }, (_, i) => maxYear - i);

	let typed = $state('');
	let fromParent = $state('');
	let open = $state(false);
	let step = $state<Step>('day');
	let draftDay = $state<number | null>(null);
	let draftMonth = $state<number | null>(null);
	let draftYear = $state<number | null>(null);

	$effect(() => {
		if (value === fromParent) return;
		fromParent = value;
		typed = value ? ymdToDdMmYyyy(value) : '';
	});

	const months = $derived(
		MONTHS.map((label, i) => ({ month: i + 1, label })).filter(
			({ month }) => draftDay === null || monthAllowsDay(month, draftDay)
		)
	);

	const years = $derived.by(() => {
		const day = draftDay;
		const month = draftMonth;
		if (day === null || month === null) return allYears;
		return allYears.filter((year) => yearAllowsDate(year, month, day));
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

	function onOpenChange(next: boolean) {
		if (next && !open) {
			step = 'day';
			const current = ymdToDateValue(value);
			draftDay = current?.day ?? null;
			draftMonth = current?.month ?? null;
			draftYear = current?.year ?? null;
		}
		open = next;
	}

	function stepTitle(s: Step): string {
		switch (s) {
			case 'day':
				return 'Jour';
			case 'month':
				return 'Mois';
			case 'year':
				return 'Année';
			default: {
				const _n: never = s;
				return _n;
			}
		}
	}

	function pickDay(day: number) {
		draftDay = day;
		step = 'month';
	}

	function pickMonth(month: number) {
		draftMonth = month;
		step = 'year';
	}

	function pickYear(year: number) {
		if (draftDay === null || draftMonth === null) return;
		const ymd = parseDdMmYyyy(`${draftDay}/${draftMonth}/${year}`);
		if (!ymd) return;
		draftYear = year;
		applyYmd(ymd);
		open = false;
	}

	function goBack() {
		if (step === 'year') step = 'month';
		else if (step === 'month') step = 'day';
	}

	function cellClass(active: boolean): string {
		return active ? 'bg-primary text-primary-foreground' : 'hover:bg-muted';
	}
</script>

<div class="relative">
	<Input
		{id}
		class="font-mono pr-16"
		inputmode="numeric"
		placeholder="jj/mm/aaaa"
		value={typed}
		oninput={(e) => onTyped(e.currentTarget.value)}
		aria-label="Date de naissance au format jj/mm/aaaa"
	/>
	<div class="absolute inset-y-0 right-0 flex items-center pr-0.5">
		{#if value}
			<Button
				variant="ghost"
				size="icon-sm"
				onclick={() => applyYmd('')}
				aria-label="Effacer la date"
			>
				<XIcon />
			</Button>
		{/if}
		<Popover.Root {open} onOpenChange={onOpenChange}>
			<Popover.Trigger>
				{#snippet child({ props })}
					<Button variant="ghost" size="icon-sm" aria-label="Choisir la date" {...props}>
						<CalendarIcon />
					</Button>
				{/snippet}
			</Popover.Trigger>
			<Popover.Content class="w-64 gap-2 p-2" align="end">
				<div class="flex items-center justify-between gap-2">
					{#if step !== 'day'}
						<Button variant="ghost" size="sm" onclick={goBack}>Retour</Button>
					{:else}
						<span></span>
					{/if}
					<p class="text-xs font-medium">{stepTitle(step)}</p>
					<span class="w-14"></span>
				</div>
				{#if step === 'day'}
					<div class="grid grid-cols-7 gap-0.5">
						{#each Array.from({ length: 31 }, (_, i) => i + 1) as day (day)}
							<button
								type="button"
								class="h-7 rounded-sm font-mono text-xs tabular-nums {cellClass(draftDay === day)}"
								onclick={() => pickDay(day)}
							>
								{day}
							</button>
						{/each}
					</div>
				{:else if step === 'month'}
					<div class="grid grid-cols-3 gap-1">
						{#each months as { month, label } (month)}
							<button
								type="button"
								class="h-8 rounded-sm px-1 text-xs capitalize {cellClass(draftMonth === month)}"
								onclick={() => pickMonth(month)}
							>
								{label}
							</button>
						{/each}
					</div>
				{:else}
					<div class="max-h-52 overflow-y-auto">
						<div class="grid grid-cols-4 gap-0.5">
							{#each years as year (year)}
								<button
									type="button"
									class="h-7 rounded-sm font-mono text-xs tabular-nums {cellClass(
										draftYear === year
									)}"
									onclick={() => pickYear(year)}
								>
									{year}
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</Popover.Content>
		</Popover.Root>
	</div>
</div>
