import { describe, expect, test } from 'bun:test';
import {
	eurosToCentimes,
	formatTarifPrix,
	moyenPaiementLabel,
	rdvClientLabel
} from '../src/lib/format';

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

describe('eurosToCentimes', () => {
	test('accepte un nombre (input type=number)', () => {
		expect(eurosToCentimes(60)).toBe(6000);
		expect(eurosToCentimes(60.5)).toBe(6050);
	});

	test('accepte une chaîne FR avec virgule et espaces', () => {
		expect(eurosToCentimes('50.00')).toBe(5000);
		expect(eurosToCentimes('60,00')).toBe(6000);
		expect(eurosToCentimes('1 234,56')).toBe(123456);
	});
});

describe('rdvClientLabel', () => {
	test('sans nom affiche Sans client', () => {
		expect(rdvClientLabel({})).toBe('Sans client');
		expect(rdvClientLabel({ client_nom: '' })).toBe('Sans client');
		expect(rdvClientLabel({ client_nom: 'Alice' })).toBe('Alice');
	});
});
