import { invoke } from '@tauri-apps/api/core';
import type {
	Client,
	Dashboard,
	Note,
	Rdv,
	RdvCreateResult,
	RdvDetail,
	SettingsPublic,
	SettingsSetInput,
	Tarif
} from './types';

export const settingsGet = () => invoke<SettingsPublic>('settings_get');

export const settingsSet = (input: SettingsSetInput) =>
	invoke<void>('settings_set', { input });

export const clientsList = () => invoke<Client[]>('clients_list');

export const clientsGet = (id: string) => invoke<Client>('clients_get', { id });

export const clientsUpsert = (p: {
	id?: string;
	nom: string;
	email?: string | null;
	telephone?: string | null;
}) => invoke<Client>('clients_upsert', p);

export const tarifsList = () => invoke<Tarif[]>('tarifs_list');

export const tarifsUpsert = (p: {
	id?: string;
	nom: string;
	duree_minutes: number;
	prix_centimes: number;
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
	client_id: string;
	tarif_id: string | null;
	debut: string;
	duree_minutes: number;
	note: string | null;
}) => invoke<RdvCreateResult>('rdv_create', p);

export const rdvUpdate = (p: {
	id: string;
	client_id: string;
	tarif_id: string | null;
	debut: string;
	duree_minutes: number;
	note: string | null;
}) => invoke<RdvCreateResult>('rdv_update', p);

export const rdvAnnuler = (id: string) => invoke<RdvCreateResult>('rdv_annuler', { id });

export const rdvDashboard = () => invoke<Dashboard>('rdv_dashboard');

export const stripeEnsureLink = (rdv_id: string) =>
	invoke<Rdv>('stripe_ensure_link', { rdv_id });

export const ntfyTest = () => invoke<void>('ntfy_test');
