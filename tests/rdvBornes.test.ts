import { describe, expect, test } from 'bun:test';
import { rdvBornes } from '../src/lib/rdvBornes';
import type { Rdv } from '../src/lib/types';

function rdv(partial: Pick<Rdv, 'id' | 'debut' | 'statut'>): Rdv {
	return {
		client_id: 'c1',
		tarif_id: null,
		duree_minutes: 60,
		jitsi_url: 'https://meet.jit.si/x',
		stripe_url: null,
		stripe_id: null,
		note: null,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...partial
	};
}

describe('rdvBornes', () => {
	const now = new Date('2026-09-15T12:00:00Z');

	test('prochain et dernier parmi les planifiés', () => {
		const list = [
			rdv({ id: 'past', debut: '2026-09-10T10:00:00Z', statut: 'planifie' }),
			rdv({ id: 'next', debut: '2026-09-20T10:00:00Z', statut: 'planifie' }),
			rdv({ id: 'later', debut: '2026-09-22T10:00:00Z', statut: 'planifie' }),
			rdv({ id: 'cancel', debut: '2026-09-11T10:00:00Z', statut: 'annule' })
		];
		const { prochain, dernier } = rdvBornes(list, now);
		expect(prochain?.id).toBe('next');
		expect(dernier?.id).toBe('past');
	});

	test('ignore les annulés pour le dernier', () => {
		const list = [rdv({ id: 'cancel', debut: '2026-09-11T10:00:00Z', statut: 'annule' })];
		expect(rdvBornes(list, now)).toEqual({ prochain: null, dernier: null });
	});

	test('un RDV à maintenant compte comme prochain', () => {
		const list = [rdv({ id: 'now', debut: now.toISOString(), statut: 'planifie' })];
		expect(rdvBornes(list, now).prochain?.id).toBe('now');
		expect(rdvBornes(list, now).dernier).toBeNull();
	});
});
