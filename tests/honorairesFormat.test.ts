import { describe, expect, test } from 'bun:test';
import { formatTarifPrix, moyenPaiementLabel } from '../src/lib/format';

describe('moyenPaiementLabel', () => {
	test('libellés', () => {
		expect(moyenPaiementLabel('especes')).toBe('Espèces');
		expect(moyenPaiementLabel('cheque')).toBe('Chèque');
		expect(moyenPaiementLabel('cb')).toBe('Carte');
		expect(moyenPaiementLabel('stripe')).toBe('Stripe');
	});
});

describe('formatTarifPrix', () => {
	test('suffixe HT ou TTC', () => {
		expect(formatTarifPrix({ prix_centimes: 5000, prix_ttc: true })).toContain('TTC');
		expect(formatTarifPrix({ prix_centimes: 5000, prix_ttc: false })).toContain('HT');
	});
});
