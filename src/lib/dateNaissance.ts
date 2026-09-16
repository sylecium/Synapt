import { CalendarDate, type DateValue } from '@internationalized/date';

function pad(n: number): string {
	return String(n).padStart(2, '0');
}

export function parseDdMmYyyy(raw: string): string | null {
	const s = raw.trim();
	const m = s.match(/^(\d{1,2})[/.\-](\d{1,2})[/.\-](\d{4})$/);
	if (!m) return null;
	const day = Number(m[1]);
	const month = Number(m[2]);
	const year = Number(m[3]);
	if (year < 1900) return null;
	try {
		const date = new CalendarDate(year, month, day);
		if (date.year !== year || date.month !== month || date.day !== day) return null;
		return `${year}-${pad(month)}-${pad(day)}`;
	} catch {
		return null;
	}
}

export function ymdToDdMmYyyy(ymd: string): string {
	const [y, m, d] = ymd.split('-');
	if (!y || !m || !d) return '';
	return `${d}/${m}/${y}`;
}

export function ymdToDateValue(ymd: string): DateValue | undefined {
	const [y, m, d] = ymd.split('-').map(Number);
	if (!y || !m || !d) return undefined;
	const date = new CalendarDate(y, m, d);
	if (date.year !== y || date.month !== m || date.day !== d) return undefined;
	return date;
}

export function dateValueToYmd(value: DateValue | undefined): string {
	if (!value) return '';
	return `${value.year}-${pad(value.month)}-${pad(value.day)}`;
}

export function daysInMonth(year: number, month: number): number {
	return new CalendarDate(year, month, 32).day;
}

export function monthAllowsDay(month: number, day: number): boolean {
	if (day <= 28) return true;
	if (day === 29) return true;
	if (day === 30) return month !== 2;
	return month === 1 || month === 3 || month === 5 || month === 7 || month === 8 || month === 10 || month === 12;
}

export function yearAllowsDate(year: number, month: number, day: number): boolean {
	return daysInMonth(year, month) >= day;
}

export function todayCalendar(): CalendarDate {
	const n = new Date();
	return new CalendarDate(n.getFullYear(), n.getMonth() + 1, n.getDate());
}
