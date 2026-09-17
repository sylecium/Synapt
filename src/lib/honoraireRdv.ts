import { honorairesGet, honorairesList } from '$lib/api';
import type { Honoraire } from '$lib/types';

export async function findHonoraireEmisForRdv(
	clientId: string | null,
	rdvId: string
): Promise<Honoraire | null> {
	if (!clientId) return null;
	const list = await honorairesList(clientId);
	for (const h of list) {
		if (h.statut !== 'emise') continue;
		const detail = await honorairesGet(h.id);
		if (detail.lignes.some((l) => l.rdv_id === rdvId && l.actif)) {
			return detail.honoraire;
		}
	}
	return null;
}
