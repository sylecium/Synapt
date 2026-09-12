import { openUrl } from '@tauri-apps/plugin-opener';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { toast } from 'svelte-sonner';
import { rdvAnnuler, stripeEnsureLink } from '$lib/api';
import { userMessage } from '$lib/errors';
import type { Rdv, SettingsPublic, Tarif } from '$lib/types';

export function stripeBlockedReason(
	rdv: Rdv,
	settings: SettingsPublic | null,
	tarifs: Tarif[]
): string {
	if (rdv.stripe_url) return '';
	if (!settings?.stripe_configured) return "Stripe n'est pas configuré.";
	if (!rdv.tarif_id) return 'Ajoutez un tarif pour demander un paiement.';
	const tarif = tarifs.find((t) => t.id === rdv.tarif_id);
	if (tarif && tarif.prix_centimes === 0) return 'Ce tarif est à 0 €.';
	return '';
}

export async function copyText(text: string, label: string): Promise<void> {
	try {
		await writeText(text);
		toast.success(`${label} copié`);
	} catch (e) {
		toast.error(userMessage(e));
	}
}

export async function openLink(url: string): Promise<void> {
	try {
		await openUrl(url);
	} catch (e) {
		toast.error(userMessage(e));
	}
}

export async function openJitsi(rdv: Rdv): Promise<void> {
	await openLink(rdv.jitsi_url);
}

export async function copyJitsi(rdv: Rdv): Promise<void> {
	await copyText(rdv.jitsi_url, 'Lien Jitsi');
}

export async function ensureStripe(rdv: Rdv): Promise<Rdv | null> {
	if (rdv.stripe_url) return rdv;
	try {
		return await stripeEnsureLink(rdv.id);
	} catch (e) {
		toast.error(userMessage(e));
		return null;
	}
}

export async function openStripe(rdv: Rdv): Promise<Rdv | null> {
	const current = await ensureStripe(rdv);
	if (current?.stripe_url) await openLink(current.stripe_url);
	return current;
}

export async function copyStripe(rdv: Rdv): Promise<Rdv | null> {
	const current = await ensureStripe(rdv);
	if (current?.stripe_url) await copyText(current.stripe_url, 'Lien de paiement');
	return current;
}

export async function cancelRdv(id: string) {
	const result = await rdvAnnuler(id);
	for (const w of result.warnings) {
		toast.error(userMessage(w));
	}
	toast.success('RDV annulé');
	return result;
}
