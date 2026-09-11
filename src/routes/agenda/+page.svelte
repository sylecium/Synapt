<script lang="ts">
	import { onMount } from 'svelte';
	import { CalendarDate, type DateValue } from '@internationalized/date';
	import { rdvList } from '$lib/api';
	import type { Rdv, RdvCreateResult } from '$lib/types';
	import { addDays, startOfWeekMonday, weekBoundsUtc } from '$lib/format';
	import WeekGrid from '$lib/components/WeekGrid.svelte';
	import RdvPanel from '$lib/components/RdvPanel.svelte';
	import RdvDialog from '$lib/components/RdvDialog.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Calendar from '$lib/components/ui/calendar/index.js';

	let weekStart = $state(startOfWeekMonday(new Date()));
	let selectedDay = $state(new Date());
	let dayView = $state(false);
	let rdvs = $state<Rdv[]>([]);
	let calendarValue = $state<DateValue | undefined>();
	let rdvDialogOpen = $state(false);
	let presetDebut = $state<string | undefined>();
	let panelOpen = $state(false);
	let panelRdvId = $state<string | null>(null);

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
		const { from, to } = weekBoundsUtc(weekStart);
		rdvs = await rdvList({ from, to });
	}

	function prevWeek() {
		weekStart = addDays(weekStart, -7);
		selectedDay = weekStart;
		calendarValue = dateToCalendar(selectedDay);
		loadRdvs();
	}

	function nextWeek() {
		weekStart = addDays(weekStart, 7);
		selectedDay = weekStart;
		calendarValue = dateToCalendar(selectedDay);
		loadRdvs();
	}

	function onSlot(isoUtc: string) {
		presetDebut = isoUtc;
		rdvDialogOpen = true;
	}

	function onRdv(r: Rdv) {
		panelRdvId = r.id;
		panelOpen = true;
	}

	function onRdvSaved(_result: RdvCreateResult) {
		rdvDialogOpen = false;
		loadRdvs();
	}

	function onPanelUpdated() {
		loadRdvs();
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex flex-wrap items-center justify-between gap-4">
		<h1 class="text-2xl font-semibold">Agenda</h1>
		<div class="flex flex-wrap gap-2">
			<Button variant={dayView ? 'outline' : 'default'} onclick={() => (dayView = false)}>
				Semaine
			</Button>
			<Button variant={dayView ? 'default' : 'outline'} onclick={() => (dayView = true)}>
				Jour
			</Button>
			<Button variant="outline" onclick={prevWeek}>Sem. préc.</Button>
			<Button variant="outline" onclick={nextWeek}>Sem. suiv.</Button>
			<Button onclick={() => (rdvDialogOpen = true)}>Nouveau RDV</Button>
		</div>
	</div>

	<div class="flex flex-wrap gap-6">
		<Calendar.Calendar
			type="single"
			bind:value={calendarValue}
			locale="fr-FR"
			class="rounded-md border p-3"
		/>
		<div class="min-w-0 flex-1">
			<WeekGrid
				startMonday={weekStart}
				{rdvs}
				{dayView}
				focusDay={selectedDay}
				{onSlot}
				{onRdv}
			/>
		</div>
	</div>
</div>

<RdvDialog
	open={rdvDialogOpen}
	{presetDebut}
	onClose={() => {
		rdvDialogOpen = false;
		presetDebut = undefined;
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
