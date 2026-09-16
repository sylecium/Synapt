<script lang="ts">
	import { onMount } from 'svelte';
	import type { Rdv } from '$lib/types';
	import { formatTime, sameLocalDay, slotUtcIso, weekDaysFromMonday } from '$lib/format';
	import RdvContextMenu from '$lib/components/RdvContextMenu.svelte';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	type Props = {
		startMonday: Date;
		rdvs: Rdv[];
		dayView?: boolean;
		focusDay?: Date;
		onSlot: (isoUtc: string) => void;
		onRdv: (r: Rdv) => void;
		onUpdated?: () => void;
	};

	let { startMonday, rdvs, dayView = false, focusDay, onSlot, onRdv, onUpdated }: Props = $props();

	let now = $state(new Date());

	const GRID_START = 8 * 60;
	const GRID_END = 20 * 60;
	const GRID_TOTAL = GRID_END - GRID_START;
	const SLOT_MINUTES = 30;
	const SLOT_HEIGHT = 32;

	const columns = $derived(
		dayView && focusDay ? [focusDay] : weekDaysFromMonday(startMonday)
	);

	const slots = $derived(
		Array.from({ length: (GRID_END - GRID_START) / SLOT_MINUTES }, (_, i) => {
			const mins = GRID_START + i * SLOT_MINUTES;
			return { hour: Math.floor(mins / 60), minute: mins % 60 };
		})
	);

	const gridHeight = $derived(slots.length * SLOT_HEIGHT);

	const nowMins = $derived(now.getHours() * 60 + now.getMinutes());
	const nowInGrid = $derived(nowMins >= GRID_START && nowMins < GRID_END);
	const nowTop = $derived(((nowMins - GRID_START) / GRID_TOTAL) * 100);

	onMount(() => {
		const id = setInterval(() => {
			now = new Date();
		}, 30_000);
		return () => clearInterval(id);
	});

	function isToday(day: Date): boolean {
		return sameLocalDay(day, new Date());
	}

	type DaySeg = { rdv: Rdv; startMins: number; endMins: number };

	function dayBounds(day: Date): { start: Date; end: Date } {
		const start = new Date(day);
		start.setHours(0, 0, 0, 0);
		const end = new Date(start);
		end.setDate(end.getDate() + 1);
		return { start, end };
	}

	function rdvEnd(rdv: Rdv): Date {
		return new Date(new Date(rdv.debut).getTime() + rdv.duree_minutes * 60_000);
	}

	function segmentOnDay(rdv: Rdv, day: Date): DaySeg | null {
		const rStart = new Date(rdv.debut);
		const rEnd = rdvEnd(rdv);
		const { start: dStart, end: dEnd } = dayBounds(day);
		const segStart = rStart > dStart ? rStart : dStart;
		const segEnd = rEnd < dEnd ? rEnd : dEnd;
		if (segEnd <= segStart) return null;
		const startMins = (segStart.getTime() - dStart.getTime()) / 60_000;
		const endMins = (segEnd.getTime() - dStart.getTime()) / 60_000;
		return { rdv, startMins, endMins };
	}

	function segsForDay(day: Date): DaySeg[] {
		const out: DaySeg[] = [];
		for (const r of rdvs) {
			const seg = segmentOnDay(r, day);
			if (seg) out.push(seg);
		}
		return out;
	}

	function clipToGrid(seg: DaySeg): { visStart: number; visEnd: number } | null {
		const visStart = Math.max(seg.startMins, GRID_START);
		const visEnd = Math.min(seg.endMins, GRID_END);
		if (visEnd <= visStart) return null;
		return { visStart, visEnd };
	}

	function rdvsOutsideForDay(day: Date): Rdv[] {
		return segsForDay(day)
			.filter((seg) => !clipToGrid(seg))
			.map((seg) => seg.rdv);
	}

	function layoutInGrid(day: Date): {
		rdv: Rdv;
		labelMins: number;
		style: string;
	}[] {
		const items = segsForDay(day)
			.map((seg) => {
				const clip = clipToGrid(seg);
				if (!clip) return null;
				return { rdv: seg.rdv, ...clip };
			})
			.filter((x): x is { rdv: Rdv; visStart: number; visEnd: number } => x !== null)
			.sort((a, b) => a.visStart - b.visStart || a.visEnd - b.visEnd);

		const clusters: (typeof items)[] = [];
		let cluster: typeof items = [];
		let clusterEnd = -1;
		for (const it of items) {
			if (cluster.length && it.visStart >= clusterEnd) {
				clusters.push(cluster);
				cluster = [];
				clusterEnd = -1;
			}
			cluster.push(it);
			clusterEnd = Math.max(clusterEnd, it.visEnd);
		}
		if (cluster.length) clusters.push(cluster);

		const placed: { rdv: Rdv; labelMins: number; style: string }[] = [];
		for (const c of clusters) {
			const colEnd: number[] = [];
			const withCol = c.map((it) => {
				let col = colEnd.findIndex((end) => end <= it.visStart);
				if (col === -1) {
					col = colEnd.length;
					colEnd.push(it.visEnd);
				} else {
					colEnd[col] = it.visEnd;
				}
				return { ...it, col };
			});
			const colCount = colEnd.length;
			for (const it of withCol) {
				const top = ((it.visStart - GRID_START) / GRID_TOTAL) * 100;
				const height = ((it.visEnd - it.visStart) / GRID_TOTAL) * 100;
				const width = 100 / colCount;
				const left = it.col * width;
				placed.push({
					rdv: it.rdv,
					labelMins: it.visStart,
					style: `top:${top}%;height:${height}%;left:calc(${left}% + 2px);width:calc(${width}% - 4px)`
				});
			}
		}
		return placed;
	}

	function dayLabel(d: Date): string {
		return new Intl.DateTimeFormat('fr-FR', { weekday: 'short', day: 'numeric' }).format(d);
	}

	function timeLabel(hour: number, minute: number): string {
		return `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
	}
</script>

<div class="overflow-x-auto">
	<div class="flex min-w-[640px]">
		<div class="w-14 shrink-0">
			<div class="h-8"></div>
			{#each slots as slot (slot.hour + ':' + slot.minute)}
				<div
					class="text-muted-foreground pr-2 text-right font-mono text-xs tabular-nums leading-8"
					style="height:{SLOT_HEIGHT}px"
				>
					{timeLabel(slot.hour, slot.minute)}
				</div>
			{/each}
		</div>

		<div
			class="grid min-w-0 flex-1"
			style="grid-template-columns: repeat({columns.length}, minmax(0, 1fr))"
		>
			{#each columns as day (day.toISOString())}
				{@const today = isToday(day)}
				<div class="border-l {today ? 'bg-primary/8' : ''}">
					<div
						class="h-8 border-b px-2 text-center text-xs font-medium {today
							? 'border-b-2 border-primary'
							: ''}"
					>
						{dayLabel(day)}
					</div>
					<div class="relative" style="height:{gridHeight}px">
						{#each slots as slot, slotIndex (day.toISOString() + slotIndex)}
							{@const iso = slotUtcIso(day, slot.hour, slot.minute)}
							<ContextMenu.Root>
								<ContextMenu.Trigger>
									{#snippet child({ props })}
										<button
											{...props}
											type="button"
											class="border-border hover:bg-muted/50 absolute inset-x-0 border-b"
											style="top:{slotIndex * SLOT_HEIGHT}px;height:{SLOT_HEIGHT}px"
											aria-label="{timeLabel(slot.hour, slot.minute)} {dayLabel(day)}"
											onclick={() => onSlot(iso)}
										></button>
									{/snippet}
								</ContextMenu.Trigger>
								<ContextMenu.Content class="w-44">
									<ContextMenu.Item onSelect={() => onSlot(iso)}>Nouveau RDV</ContextMenu.Item>
								</ContextMenu.Content>
							</ContextMenu.Root>
						{/each}
						{#if today && nowInGrid}
							<div
								class="pointer-events-none absolute inset-x-0 z-20 -translate-y-1/2"
								style="top:{nowTop}%"
							>
								<div class="flex items-center">
									<span
										class="bg-destructive text-destructive-foreground ml-0.5 rounded px-1 font-mono text-[10px] leading-4 tabular-nums"
									>
										{timeLabel(now.getHours(), now.getMinutes())}
									</span>
									<span class="bg-destructive size-1.5 shrink-0 rounded-full"></span>
									<div class="bg-destructive h-px flex-1"></div>
								</div>
							</div>
						{/if}
						{#each layoutInGrid(day) as block (`${block.rdv.id}:${day.toISOString()}`)}
							<RdvContextMenu rdv={block.rdv} onOpen={() => onRdv(block.rdv)} {onUpdated}>
								{#snippet children(props)}
									<button
										{...props}
										type="button"
										class="bg-primary text-primary-foreground absolute z-10 overflow-hidden rounded-md px-1 py-0.5 text-left text-xs hover:opacity-90"
										style={block.style}
										onclick={(e) => {
											e.stopPropagation();
											onRdv(block.rdv);
										}}
									>
										<span class="font-medium">{block.rdv.client_nom}</span>
										<span class="font-mono tabular-nums opacity-80">{timeLabel(Math.floor(block.labelMins / 60), block.labelMins % 60)}</span>
									</button>
								{/snippet}
							</RdvContextMenu>
						{/each}
					</div>
					{#if rdvsOutsideForDay(day).length > 0}
						<div class="flex flex-col gap-1 border-t px-1 py-2">
							<p class="text-muted-foreground text-[10px] font-medium uppercase">Hors plage</p>
							{#each rdvsOutsideForDay(day) as rdv (rdv.id)}
								<RdvContextMenu {rdv} onOpen={() => onRdv(rdv)} {onUpdated}>
									{#snippet children(props)}
										<button
											{...props}
											type="button"
											class="bg-warn-bg text-warn hover:opacity-90 rounded-md px-2 py-1 text-left text-xs"
											onclick={() => onRdv(rdv)}
										>
											<span class="font-medium">{rdv.client_nom}</span>
											<span class="ml-1 font-mono tabular-nums">{formatTime(rdv.debut)}</span>
										</button>
									{/snippet}
								</RdvContextMenu>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
</div>
