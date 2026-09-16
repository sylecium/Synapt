import { CalendarDate } from '@internationalized/date';

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
