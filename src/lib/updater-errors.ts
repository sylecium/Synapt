function rawMessage(err: unknown): string {
	if (typeof err === 'string') return err;
	if (err instanceof Error) return err.message;
	if (err && typeof err === 'object' && 'message' in err) {
		return String((err as { message: unknown }).message);
	}
	return '';
}

export function installUpdateMessage(err: unknown): string {
	const lower = rawMessage(err).toLowerCase();
	if (
		lower.includes('401') ||
		lower.includes('403') ||
		lower.includes('unauthorized') ||
		lower.includes('forbidden')
	) {
		return 'Accès aux mises à jour refusé.';
	}
	if (
		lower.includes('signature') ||
		lower.includes('invalid updater') ||
		lower.includes('not a valid deb')
	) {
		return 'La mise à jour n’est pas fiable, installation annulée.';
	}
	if (
		lower.includes('packageinstallfailed') ||
		lower.includes('pkexec') ||
		lower.includes('dpkg') ||
		lower.includes('cancelled') ||
		lower.includes('canceled')
	) {
		return 'Installation de la mise à jour annulée.';
	}
	return 'Impossible de télécharger la mise à jour.';
}
