import { describe, expect, test } from 'bun:test';
import { nextRdv, remainingRdvs } from '../src/lib/dashboardStats';

const rdv = (debut: string) => ({ debut, id: debut });

describe('remainingRdvs', () => {
	test('garde seulement les RDV dont l’heure n’est pas passée', () => {
		const now = new Date('2026-09-12T12:00:00.000Z');
		const left = remainingRdvs(
			[rdv('2026-09-12T09:00:00.000Z'), rdv('2026-09-12T14:00:00.000Z')],
			now
		);
		expect(left.map((r) => r.debut)).toEqual(['2026-09-12T14:00:00.000Z']);
	});
});

describe('nextRdv', () => {
	test('prend le prochain du jour avant les à venir', () => {
		const now = new Date('2026-09-12T12:00:00.000Z');
		const next = nextRdv(
			[rdv('2026-09-12T09:00:00.000Z'), rdv('2026-09-12T15:30:00.000Z')],
			[rdv('2026-09-13T10:00:00.000Z')],
			now
		);
		expect(next?.debut).toBe('2026-09-12T15:30:00.000Z');
	});

	test('bascule sur le premier à venir si plus rien aujourd’hui', () => {
		const now = new Date('2026-09-12T18:00:00.000Z');
		const next = nextRdv(
			[rdv('2026-09-12T09:00:00.000Z')],
			[rdv('2026-09-13T10:00:00.000Z')],
			now
		);
		expect(next?.debut).toBe('2026-09-13T10:00:00.000Z');
	});

	test('null s’il n’y a rien', () => {
		expect(nextRdv([], [], new Date('2026-09-12T12:00:00.000Z'))).toBeNull();
	});
});
