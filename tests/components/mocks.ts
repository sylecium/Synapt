import type { Client, HonoraireDetail, Rdv, Tarif } from '$lib/types';

export const mockClients: Client[] = [
	{
		id: 'client-1',
		nom: 'Marie Curie',
		email: 'marie.curie@example.com',
		telephone: '0601020304',
		statut: 'en_cours',
		memo: 'Première consultation',
		tarif_id: 'tarif-1',
		date_naissance: '1867-11-07',
		urgence_nom: 'Pierre Curie',
		urgence_telephone: '0605060708',
		orientation: 'medecin',
		frequence: 'hebdo',
		adresse: '1 rue Pierre et Marie Curie\n75005 Paris',
		created_at: '2026-09-01T10:00:00Z',
		updated_at: '2026-09-01T10:00:00Z'
	},
	{
		id: 'client-2',
		nom: 'Louis Pasteur',
		email: 'louis.pasteur@example.com',
		telephone: '0611223344',
		statut: 'pause',
		memo: null,
		tarif_id: null,
		date_naissance: '1822-12-27',
		urgence_nom: null,
		urgence_telephone: null,
		orientation: null,
		frequence: null,
		adresse: null,
		created_at: '2026-09-02T10:00:00Z',
		updated_at: '2026-09-02T10:00:00Z'
	}
];

export const mockTarifs: Tarif[] = [
	{
		id: 'tarif-1',
		nom: 'Consultation standard',
		duree_minutes: 60,
		prix_centimes: 6000,
		prix_ttc: true,
		actif: true,
		created_at: '2026-09-01T10:00:00Z',
		updated_at: '2026-09-01T10:00:00Z'
	},
	{
		id: 'tarif-2',
		nom: 'Suivi court',
		duree_minutes: 30,
		prix_centimes: 3500,
		prix_ttc: true,
		actif: true,
		created_at: '2026-09-01T10:00:00Z',
		updated_at: '2026-09-01T10:00:00Z'
	}
];

export const mockRdvs: Rdv[] = [
	{
		id: 'rdv-1',
		client_id: 'client-1',
		client_nom: 'Marie Curie',
		tarif_id: 'tarif-1',
		tarif_nom: 'Consultation standard',
		debut: '2026-10-15T09:00:00Z',
		duree_minutes: 60,
		jitsi_url: 'https://meet.jit.si/synapt-test-1',
		stripe_url: null,
		stripe_id: null,
		note: 'Note de séance',
		statut: 'planifie',
		created_at: '2026-09-01T10:00:00Z',
		updated_at: '2026-09-01T10:00:00Z'
	},
	{
		id: 'rdv-2',
		client_id: 'client-1',
		client_nom: 'Marie Curie',
		tarif_id: 'tarif-1',
		tarif_nom: 'Consultation standard',
		debut: '2026-10-22T09:00:00Z',
		duree_minutes: 60,
		jitsi_url: 'https://meet.jit.si/synapt-test-2',
		stripe_url: null,
		stripe_id: null,
		note: null,
		statut: 'planifie',
		created_at: '2026-09-01T10:00:00Z',
		updated_at: '2026-09-01T10:00:00Z'
	}
];

export const mockHonoraireDetail: HonoraireDetail = {
	honoraire: {
		id: 'hon-1',
		numero: 'NH-2026-001',
		client_id: 'client-1',
		client_nom: 'Marie Curie',
		client_date_naissance: '1867-11-07',
		client_adresse: '1 rue Pierre et Marie Curie\n75005 Paris',
		cabinet_nom: 'Cabinet Psy Test',
		cabinet_adresse: '10 rue de la Paix 75002 Paris',
		cabinet_telephone: '0102030405',
		cabinet_email: 'psy@cabinet.fr',
		cabinet_siret: '12345678900012',
		mention_tva: 'TVA non applicable, art. 293 B du CGI',
		moyen_paiement: 'especes',
		total_ht_centimes: 6000,
		total_tva_centimes: 0,
		total_ttc_centimes: 6000,
		statut: 'emise',
		pdf_relatif: 'NH-2026-001.pdf',
		created_at: '2026-10-15T10:00:00Z'
	},
	lignes: [
		{
			id: 'ligne-1',
			honoraire_id: 'hon-1',
			rdv_id: 'rdv-1',
			description: 'Consultation du 15/10/2026 11:00',
			prix_unitaire_centimes: 6000,
			taux_tva: 0,
			montant_ht_centimes: 6000,
			montant_tva_centimes: 0,
			montant_ttc_centimes: 6000,
			ordre: 1
		}
	]
};
