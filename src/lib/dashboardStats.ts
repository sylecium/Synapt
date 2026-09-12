export function remainingRdvs<T extends { debut: string }>(rdvs: T[], now: Date): T[] {
	const t = now.getTime();
	return rdvs.filter((r) => new Date(r.debut).getTime() > t);
}

export function nextRdv<T extends { debut: string }>(today: T[], upcoming: T[], now: Date): T | null {
	return remainingRdvs(today, now)[0] ?? upcoming[0] ?? null;
}
