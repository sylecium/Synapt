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
	adresse: string | null;
	created_at: string;
	updated_at: string;
};

export type Tarif = {
	id: string;
	nom: string;
	duree_minutes: number;
	prix_centimes: number;
	prix_ttc: boolean;
	actif: boolean;
	created_at: string;
	updated_at: string;
};

export type Rdv = {
	id: string;
	client_id: string | null;
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

export type Dashboard = {
	aujourdhui: Rdv[];
	a_venir: Rdv[];
	clients_count: number;
	tarifs_count: number;
	week_count: number;
};

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
	cabinet_nom: string;
	cabinet_adresse: string;
	cabinet_telephone: string;
	cabinet_email: string;
	cabinet_siret: string;
	mention_tva: string;
	prefixe_numero: string;
};

export type SettingsSetInput = {
	ntfy_serveur: string;
	ntfy_topic: string;
	ntfy_token: string;
	ntfy_token_clear: boolean;
	rappel_24h: boolean;
	rappel_1h: boolean;
	stripe_secret_key: string;
	stripe_secret_clear: boolean;
	cabinet_nom: string;
	cabinet_adresse: string;
	cabinet_telephone: string;
	cabinet_email: string;
	cabinet_siret: string;
	mention_tva: string;
	prefixe_numero: string;
};

export type MoyenPaiement = 'especes' | 'cheque' | 'cb' | 'stripe';
export type HonoraireStatut = 'emise' | 'annulee';

export type Honoraire = {
	id: string;
	numero: string;
	client_id: string;
	client_nom: string;
	client_date_naissance: string | null;
	client_adresse: string | null;
	cabinet_nom: string;
	cabinet_adresse: string | null;
	cabinet_telephone: string | null;
	cabinet_email: string | null;
	cabinet_siret: string | null;
	mention_tva: string;
	moyen_paiement: MoyenPaiement;
	statut: HonoraireStatut;
	total_centimes: number;
	annee: number;
	seq: number;
	pdf_relatif: string;
	created_at: string;
};

export type HonoraireLigne = {
	id: string;
	honoraire_id: string;
	rdv_id: string;
	debut: string;
	duree_minutes: number;
	tarif_nom: string;
	prix_centimes: number;
	actif: boolean;
};

export type HonoraireDetail = {
	honoraire: Honoraire;
	lignes: HonoraireLigne[];
};
