import { expect, test } from 'bun:test';
import { installUpdateMessage } from './updater-errors';

test('accès GitHub refusé', () => {
	expect(installUpdateMessage('401 Unauthorized')).toBe('Accès aux mises à jour refusé.');
	expect(installUpdateMessage('status code 403')).toBe('Accès aux mises à jour refusé.');
});

test('signature invalide', () => {
	expect(installUpdateMessage('signature verification failed')).toBe(
		'La mise à jour n’est pas fiable, installation annulée.'
	);
	expect(installUpdateMessage('invalid updater binary format')).toBe(
		'La mise à jour n’est pas fiable, installation annulée.'
	);
});

test('Polkit ou dpkg', () => {
	expect(installUpdateMessage('PackageInstallFailed')).toBe(
		'Installation de la mise à jour annulée.'
	);
	expect(installUpdateMessage('pkexec cancelled')).toBe(
		'Installation de la mise à jour annulée.'
	);
});

test('réseau par défaut', () => {
	expect(installUpdateMessage(new Error('Failed to fetch'))).toBe(
		'Impossible de télécharger la mise à jour.'
	);
	expect(installUpdateMessage('')).toBe('Impossible de télécharger la mise à jour.');
});
