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
