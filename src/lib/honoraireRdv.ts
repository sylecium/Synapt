import { honorairesGetForRdv } from '$lib/api';
import type { Honoraire } from '$lib/types';

export async function findHonoraireEmisForRdv(
	clientId: string | null,
	rdvId: string
): Promise<Honoraire | null> {
	if (!clientId || !rdvId) return null;
	return await honorairesGetForRdv(rdvId);
}
