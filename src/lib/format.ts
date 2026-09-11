export function formatTime(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', { timeStyle: 'short' }).format(new Date(iso));
}

export function startOfWeekMonday(d: Date): Date {
	const date = new Date(d);
	date.setHours(0, 0, 0, 0);
	const day = date.getDay();
	const diff = day === 0 ? -6 : 1 - day;
	date.setDate(date.getDate() + diff);
	return date;
}

export function addDays(d: Date, n: number): Date {
	const date = new Date(d);
	date.setDate(date.getDate() + n);
	return date;
}

export function formatWeekLabel(startMonday: Date): string {
	const end = addDays(startMonday, 6);
	const dayFmt = new Intl.DateTimeFormat('fr-FR', { day: 'numeric' });
	const endFmt = new Intl.DateTimeFormat('fr-FR', { day: 'numeric', month: 'short' });
	return `${dayFmt.format(startMonday)}–${endFmt.format(end)}`;
}

export function formatDayLabel(d: Date): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'short',
		day: 'numeric',
		month: 'short'
	}).format(d);
}

export function sameLocalDay(a: Date, b: Date): boolean {
	return (
		a.getFullYear() === b.getFullYear() &&
		a.getMonth() === b.getMonth() &&
		a.getDate() === b.getDate()
	);
}

export function weekDaysFromMonday(startMonday: Date): Date[] {
	return Array.from({ length: 7 }, (_, i) => addDays(startMonday, i));
}

export function weekBoundsUtc(startMonday: Date): { from: string; to: string } {
	const from = new Date(startMonday);
	from.setHours(0, 0, 0, 0);
	const to = addDays(startMonday, 7);
	to.setHours(0, 0, 0, 0);
	return { from: from.toISOString(), to: to.toISOString() };
}

export function slotUtcIso(day: Date, hour: number, minute: number): string {
	const d = new Date(day);
	d.setHours(hour, minute, 0, 0);
	return d.toISOString();
}

export function formatCentimes(centimes: number): string {
	return new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'EUR' }).format(
		centimes / 100
	);
}

export function formatDateTime(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', { dateStyle: 'short', timeStyle: 'short' }).format(
		new Date(iso)
	);
}

export function formatNoteListDate(iso: string): string {
	const d = new Date(iso);
	if (sameLocalDay(d, new Date())) {
		return formatTime(iso);
	}
	return new Intl.DateTimeFormat('fr-FR', { day: 'numeric', month: 'short' }).format(d);
}

export function utcIsoToLocalDatetime(iso: string): string {
	const d = new Date(iso);
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function localDatetimeToUtcIso(localValue: string): string {
	return new Date(localValue).toISOString();
}

export function eurosToCentimes(euros: string): number {
	const n = Number.parseFloat(euros.replace(',', '.'));
	if (Number.isNaN(n)) return 0;
	return Math.round(n * 100);
}

export function centimesToEuros(centimes: number): string {
	return (centimes / 100).toFixed(2);
}
