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
