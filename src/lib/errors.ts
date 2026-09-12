const FALLBACK = "Impossible de terminer l'action. Réessayez.";

function rawMessage(err: unknown): string {
	if (typeof err === 'string') return err;
	if (err instanceof Error) return err.message;
	if (err && typeof err === 'object' && 'message' in err) {
		return String((err as { message: unknown }).message);
	}
	return '';
}

export function userMessage(err: unknown): string {
	const raw = rawMessage(err).trim();
	if (!raw) return FALLBACK;

	const lower = raw.toLowerCase();

	if (lower.includes('chevauche')) {
		return 'Ce créneau chevauche un autre rendez-vous.';
	}
	if (lower.includes('duree_minutes') || lower.includes('doit etre positif')) {
		return 'La durée doit être supérieure à 0 minute.';
	}
	if (lower.includes('rdv non modifiable')) {
		return 'Ce rendez-vous ne peut plus être modifié.';
	}
	if (lower.includes('rdv introuvable')) {
		return 'Rendez-vous introuvable.';
	}
	if (lower.includes('client introuvable')) {
		return 'Client introuvable.';
	}
	if (lower.includes('tarif introuvable')) {
		return 'Tarif introuvable.';
	}
	if (lower.includes('note introuvable')) {
		return 'Note introuvable.';
	}
	if (lower.includes('stripe')) {
		return "Le lien de paiement n'a pas pu être créé. Vérifiez la clé Stripe dans les réglages.";
	}
	if (lower.includes('ntfy')) {
		return "La notification n'a pas pu être envoyée. Vérifiez ntfy dans les réglages.";
	}
	if (
		lower.includes('invalid args') ||
		lower.includes('missing required key') ||
		lower.includes('error invoking') ||
		lower.includes('ipc') ||
		lower.includes('sqlite') ||
		lower.includes('foreign key') ||
		/command ['`]/.test(lower)
	) {
		return FALLBACK;
	}

	if (raw.length < 120 && !/[`{}]/.test(raw) && !/error:/i.test(raw)) {
		return raw;
	}

	return FALLBACK;
}
