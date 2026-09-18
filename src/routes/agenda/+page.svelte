<script lang="ts">
	import { onMount } from 'svelte';
	import { CalendarDate, type DateValue } from '@internationalized/date';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import { toast } from 'svelte-sonner';
	import { rdvList } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { Rdv, RdvCreateResult } from '$lib/types';
	import {
		addDays,
		formatDayLabel,
		formatWeekLabel,
		startOfWeekMonday,
		weekBoundsUtc
	} from '$lib/format';
	import WeekGrid from '$lib/components/WeekGrid.svelte';
	import RdvPanel from '$lib/components/RdvPanel.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Calendar from '$lib/components/ui/calendar/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';

	let weekStart = $state(startOfWeekMonday(new Date()));
	let selectedDay = $state(new Date());
	let dayView = $state(false);
	let rdvs = $state<Rdv[]>([]);
	let calendarValue = $state<DateValue | undefined>();
	let rdvDialogOpen = $state(false);
	let presetDebut = $state<string | undefined>();
	let panelOpen = $state(false);
	let panelRdvId = $state<string | null>(null);
	let editRdvId = $state<string | null>(null);
	let loading = $state(true);

	const navLabel = $derived(
		dayView ? formatDayLabel(selectedDay) : formatWeekLabel(weekStart)
	);

	onMount(async () => {
		calendarValue = dateToCalendar(selectedDay);
		await loadRdvs();
	});

	$effect(() => {
		if (!calendarValue) return;
		const date = calendarToDate(calendarValue);
		if (
			date.getFullYear() === selectedDay.getFullYear() &&
			date.getMonth() === selectedDay.getMonth() &&
			date.getDate() === selectedDay.getDate()
		) {
			return;
		}
		selectedDay = date;
		weekStart = startOfWeekMonday(date);
		loadRdvs();
	});

	function dateToCalendar(d: Date): CalendarDate {
		return new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate());
	}

	function calendarToDate(c: DateValue): Date {
		return new Date(c.year, c.month - 1, c.day);
	}

	async function loadRdvs() {
		try {
			const { from, to } = weekBoundsUtc(weekStart);
			rdvs = await rdvList({ from, to });
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			loading = false;
		}
	}

	function goToday() {
		const today = new Date();
		const cur = calendarValue ? calendarToDate(calendarValue) : null;
		if (
			cur &&
			cur.getFullYear() === today.getFullYear() &&
			cur.getMonth() === today.getMonth() &&
			cur.getDate() === today.getDate()
		) {
			void loadRdvs();
		} else {
			calendarValue = dateToCalendar(today);
		}
	}

	function prev() {
		const base = dayView ? selectedDay : weekStart;
		calendarValue = dateToCalendar(addDays(base, dayView ? -1 : -7));
	}

	function next() {
		const base = dayView ? selectedDay : weekStart;
		calendarValue = dateToCalendar(addDays(base, dayView ? 1 : 7));
	}

	function onSlot(isoUtc: string) {
		presetDebut = isoUtc;
		rdvDialogOpen = true;
	}

	function onRdv(r: Rdv) {
		panelRdvId = r.id;
		panelOpen = true;
	}

	function openEditRdv(r: Rdv) {
		editRdvId = r.id;
		presetDebut = undefined;
		rdvDialogOpen = true;
	}

	function onRdvSaved(_result: RdvCreateResult) {
		rdvDialogOpen = false;
		presetDebut = undefined;
		editRdvId = null;
		loadRdvs();
	}

	function onPanelUpdated() {
		loadRdvs();
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Agenda" />

	<div class="flex flex-wrap items-center gap-2">
		<Button variant="outline" onclick={goToday}>Aujourd'hui</Button>
		<Button variant="outline" size="icon" onclick={prev} aria-label="Précédent">
			<ChevronLeftIcon class="size-4" />
		</Button>
		<span class="min-w-28 text-center text-sm font-medium">{navLabel}</span>
		<Button variant="outline" size="icon" onclick={next} aria-label="Suivant">
			<ChevronRightIcon class="size-4" />
		</Button>

		<Button variant={dayView ? 'outline' : 'default'} onclick={() => (dayView = false)}>
			Semaine
		</Button>
		<Button variant={dayView ? 'default' : 'outline'} onclick={() => (dayView = true)}>
			Jour
		</Button>

		<Popover.Root>
			<Popover.Trigger>
				{#snippet child({ props })}
					<Button variant="outline" size="icon" {...props} aria-label="Choisir une date">
						<CalendarIcon class="size-4" />
					</Button>
				{/snippet}
			</Popover.Trigger>
			<Popover.Content class="w-auto p-0">
				<Calendar.Calendar type="single" bind:value={calendarValue} locale="fr-FR" />
			</Popover.Content>
		</Popover.Root>

		<Button
			onclick={() => {
				presetDebut = undefined;
				rdvDialogOpen = true;
			}}
		>
			Nouveau RDV
		</Button>
	</div>

	{#if loading}
		<Skeleton class="h-[32rem]" />
	{:else}
		<WeekGrid
			startMonday={weekStart}
			{rdvs}
			{dayView}
			focusDay={selectedDay}
			{onSlot}
			{onRdv}
			onEdit={openEditRdv}
			onUpdated={loadRdvs}
		/>
	{/if}
</div>

<RdvDialog
	open={rdvDialogOpen}
	rdvId={editRdvId ?? undefined}
	{presetDebut}
	onClose={() => {
		rdvDialogOpen = false;
		presetDebut = undefined;
		editRdvId = null;
	}}
	onSaved={onRdvSaved}
/>

<RdvPanel
	bind:open={panelOpen}
	rdvId={panelRdvId}
	onClose={() => {
		panelRdvId = null;
	}}
	onUpdated={onPanelUpdated}
/>
