import type { Rdv } from './types';

export function rdvBornes(
	rdvs: Rdv[],
	now: Date = new Date()
): { prochain: Rdv | null; dernier: Rdv | null } {
	const t = now.getTime();
	const planifies = rdvs.filter((r) => r.statut === 'planifie');
	const upcoming = planifies
		.filter((r) => new Date(r.debut).getTime() >= t)
		.sort((a, b) => a.debut.localeCompare(b.debut));
	const past = planifies
		.filter((r) => new Date(r.debut).getTime() < t)
		.sort((a, b) => a.debut.localeCompare(b.debut));
	return {
		prochain: upcoming[0] ?? null,
		dernier: past[past.length - 1] ?? null
	};
}
