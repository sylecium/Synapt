import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { userMessage } from './errors';
import type {
	Client,
	Dashboard,
	Honoraire,
	HonoraireDetail,
	MoyenPaiement,
	Note,
	Rdv,
	RdvCreateResult,
	RdvDetail,
	SettingsPublic,
	SettingsSetInput,
	Tarif
} from './types';

function invoke<T>(cmd: string, args?: object): Promise<T> {
	return tauriInvoke<T>(cmd, args as Record<string, unknown>).catch((e) =>
		Promise.reject(userMessage(e))
	);
}

export const settingsGet = () => invoke<SettingsPublic>('settings_get');

export const settingsSet = (input: SettingsSetInput) =>
	invoke<void>('settings_set', { input });

export const clientsList = () => invoke<Client[]>('clients_list');

export const clientsGet = (id: string) => invoke<Client>('clients_get', { id });

export const clientsDelete = (id: string) => invoke<void>('clients_delete', { id });

export const clientsUpsert = (p: {
	id?: string;
	nom: string;
	email?: string | null;
	telephone?: string | null;
	statut?: string | null;
	memo?: string | null;
	tarif_id?: string | null;
	date_naissance?: string | null;
	urgence_nom?: string | null;
	urgence_telephone?: string | null;
	orientation?: string | null;
	frequence?: string | null;
	adresse?: string | null;
}) => invoke<Client>('clients_upsert', p);

export const tarifsList = () => invoke<Tarif[]>('tarifs_list');

export const tarifsUpsert = (p: {
	id?: string;
	nom: string;
	duree_minutes: number;
	prix_centimes: number;
	prix_ttc: boolean;
}) => invoke<Tarif>('tarifs_upsert', p);

export const tarifsSetActif = (id: string, actif: boolean) =>
	invoke<Tarif>('tarifs_set_actif', { id, actif });

export const notesList = (p?: { client_id?: string; perso?: boolean }) =>
	invoke<Note[]>('notes_list', {
		client_id: p?.client_id ?? null,
		perso: p?.perso ?? false
	});

export const notesUpsert = (p: { id?: string; client_id?: string | null; corps: string }) =>
	invoke<Note>('notes_upsert', p);

export const notesDelete = (id: string) => invoke<void>('notes_delete', { id });

export const rdvList = (p: { from?: string; to?: string; client_id?: string }) =>
	invoke<Rdv[]>('rdv_list', {
		from: p.from ?? null,
		to: p.to ?? null,
		client_id: p.client_id ?? null
	});

export const rdvGet = (id: string) => invoke<RdvDetail>('rdv_get', { id });

export const rdvCreate = (p: {
	client_id: string | null;
	tarif_id: string | null;
	debut: string;
	duree_minutes: number;
	note: string | null;
}) => invoke<RdvCreateResult>('rdv_create', p);

export const rdvUpdate = (p: {
	id: string;
	client_id: string | null;
	tarif_id: string | null;
	debut: string;
	duree_minutes: number;
	note: string | null;
}) => invoke<RdvCreateResult>('rdv_update', p);

export const rdvAnnuler = (id: string) => invoke<RdvCreateResult>('rdv_annuler', { id });

export const rdvSetNote = (id: string, note: string | null) =>
	invoke<Rdv>('rdv_set_note', { id, note });

export const rdvDashboard = () => invoke<Dashboard>('rdv_dashboard');

export const stripeEnsureLink = (rdv_id: string) =>
	invoke<Rdv>('stripe_ensure_link', { rdv_id });

export const ntfyTest = () => invoke<void>('ntfy_test');

export const honorairesList = (client_id?: string) =>
	invoke<Honoraire[]>('honoraires_list', { client_id: client_id ?? null });

export const honorairesGet = (id: string) => invoke<HonoraireDetail>('honoraires_get', { id });

export const honorairesCreate = (p: { rdv_ids: string[]; moyen_paiement: MoyenPaiement }) =>
	invoke<HonoraireDetail>('honoraires_create', p);

export const honorairesOuvrir = (id: string) => invoke<string>('honoraires_ouvrir', { id });

export const honorairesAnnuler = (id: string) => invoke<HonoraireDetail>('honoraires_annuler', { id });

export const honorairesRdvsDisponibles = (client_id: string) =>
	invoke<Rdv[]>('honoraires_rdvs_disponibles', { client_id });
