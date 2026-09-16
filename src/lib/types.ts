export type ClientStatut = 'en_cours' | 'pause' | 'termine';
export type ClientOrientation = 'medecin' | 'reco' | 'lui_meme';
export type ClientFrequence = 'hebdo' | 'bimensuel' | 'a_la_demande';

export type Client = {
	id: string;
	nom: string;
	email: string | null;
	telephone: string | null;
	statut: ClientStatut;
	memo: string | null;
	tarif_id: string | null;
	date_naissance: string | null;
	urgence_nom: string | null;
	urgence_telephone: string | null;
	orientation: ClientOrientation | null;
	frequence: ClientFrequence | null;
	created_at: string;
	updated_at: string;
};

export type Tarif = {
	id: string;
	nom: string;
	duree_minutes: number;
	prix_centimes: number;
	actif: boolean;
	created_at: string;
	updated_at: string;
};

export type Rdv = {
	id: string;
	client_id: string;
	tarif_id: string | null;
	debut: string;
	duree_minutes: number;
	jitsi_url: string;
	stripe_url: string | null;
	stripe_id: string | null;
	note: string | null;
	statut: 'planifie' | 'annule';
	created_at: string;
	updated_at: string;
	client_nom?: string;
	tarif_nom?: string;
};

export type RdvCreateResult = { rdv: Rdv; warnings: string[] };

export type RappelNtfy = {
	id: string;
	rdv_id: string;
	type: '24h' | '1h';
	ntfy_id: string | null;
	echeance: string;
	etat: 'programme' | 'annule';
};

export type RdvDetail = { rdv: Rdv; rappels: RappelNtfy[] };

export type Dashboard = { aujourdhui: Rdv[]; a_venir: Rdv[] };

export type Note = {
	id: string;
	client_id: string | null;
	corps: string;
	created_at: string;
	updated_at: string;
};

export type SettingsPublic = {
	ntfy_serveur: string;
	ntfy_topic: string;
	ntfy_token_configured: boolean;
	ntfy_token_last4: string;
	rappel_24h: boolean;
	rappel_1h: boolean;
	stripe_configured: boolean;
	stripe_last4: string;
};

export type SettingsSetInput = {
	ntfy_serveur: string;
	ntfy_topic: string;
	ntfy_token: string;
	rappel_24h: boolean;
	rappel_1h: boolean;
	stripe_secret_key: string;
};
