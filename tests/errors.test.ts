import { describe, expect, test } from 'bun:test';
import { userMessage } from '../src/lib/errors';

describe('userMessage', () => {
	test('base de données verrouillée', () => {
		const msg = "Impossible d'enregistrer pour le moment. Réessayez.";
		expect(userMessage('database is locked')).toBe(msg);
		expect(userMessage('database is busy')).toBe(msg);
		expect(userMessage('SQLITE error: database table is locked')).toBe(msg);
	});

	test('sans jargon sqlite générique', () => {
		expect(userMessage('sqlite constraint failed')).toBe(
			"Impossible de terminer l'action. Réessayez."
		);
	});

	test('messages honoraires spec', () => {
		const msg = 'Indiquez le nom du cabinet dans les réglages.';
		expect(userMessage(msg)).toBe(msg);
		expect(userMessage("Ce client a encore des notes d'honoraires.")).toBe(
			"Ce client a encore des notes d'honoraires."
		);
	});
});
