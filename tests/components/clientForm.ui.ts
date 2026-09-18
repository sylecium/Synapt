import { describe, expect, test } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ClientForm from '$lib/components/ClientForm.svelte';
import { mockTarifs } from './mocks';

describe('ClientForm', () => {
	test('rendu initial en mode création avec champs identité de base', () => {
		render(ClientForm, {
			props: {
				variant: 'create',
				tarifs: mockTarifs,
				nom: '',
				email: '',
				telephone: '',
				statut: 'en_cours',
				memo: '',
				tarifId: '',
				dateNaissance: '',
				urgenceNom: '',
				urgenceTelephone: '',
				orientation: '',
				frequence: '',
				adresse: ''
			}
		});

		expect(screen.getByLabelText('Nom', { selector: '#client-nom' })).toBeInTheDocument();
		expect(screen.getByLabelText('Email', { selector: '#client-email' })).toBeInTheDocument();
		expect(screen.getByLabelText('Téléphone', { selector: '#client-telephone' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Ajouter des champs/i })).toBeInTheDocument();
		// En mode création initiale, les extras sont masqués
		expect(screen.queryByLabelText('Date de naissance')).not.toBeInTheDocument();
		expect(screen.queryByLabelText(/Adresse/i)).not.toBeInTheDocument();
	});

	test('rendu en mode édition affiche toutes les sections', () => {
		render(ClientForm, {
			props: {
				variant: 'edit',
				tarifs: mockTarifs,
				nom: 'Marie Curie',
				email: 'marie@curie.fr',
				telephone: '0601020304',
				statut: 'en_cours',
				memo: 'Note de suivi',
				tarifId: 'tarif-1',
				dateNaissance: '1867-11-07',
				urgenceNom: 'Pierre Curie',
				urgenceTelephone: '0605060708',
				orientation: 'medecin',
				frequence: 'hebdo',
				adresse: '1 rue Curie'
			}
		});

		expect(screen.getByLabelText('Nom', { selector: '#nom' })).toHaveValue('Marie Curie');
		expect(screen.getByLabelText('Email', { selector: '#email' })).toHaveValue('marie@curie.fr');
		expect(screen.getByLabelText('Téléphone', { selector: '#telephone' })).toHaveValue('0601020304');
		expect(screen.getByLabelText('Date de naissance', { selector: '#date-naissance' })).toHaveValue('07/11/1867');
		expect(screen.getByLabelText('Adresse', { selector: '#adresse' })).toHaveValue('1 rue Curie');
		expect(screen.getByLabelText('Nom', { selector: '#urgence-nom' })).toHaveValue('Pierre Curie');
		expect(screen.getByLabelText('Téléphone', { selector: '#urgence-tel' })).toHaveValue('0605060708');
		expect(screen.getByLabelText('Mémo', { selector: '#memo' })).toHaveValue('Note de suivi');
	});

	test('saisie et mise à jour des champs de texte en mode création', async () => {
		render(ClientForm, {
			props: {
				variant: 'create',
				tarifs: mockTarifs,
				nom: '',
				email: '',
				telephone: '',
				statut: 'en_cours',
				memo: '',
				tarifId: '',
				dateNaissance: '',
				urgenceNom: '',
				urgenceTelephone: '',
				orientation: '',
				frequence: '',
				adresse: ''
			}
		});

		const nomInput = screen.getByLabelText('Nom', { selector: '#client-nom' });
		await fireEvent.input(nomInput, { target: { value: 'Pasteur' } });
		expect(nomInput).toHaveValue('Pasteur');

		const emailInput = screen.getByLabelText('Email', { selector: '#client-email' });
		await fireEvent.input(emailInput, { target: { value: 'louis@pasteur.fr' } });
		expect(emailInput).toHaveValue('louis@pasteur.fr');
	});

	test('saisie de la date de naissance via DateNaissanceField en mode édition', async () => {
		render(ClientForm, {
			props: {
				variant: 'edit',
				tarifs: mockTarifs,
				nom: 'Jean Moulin',
				email: '',
				telephone: '',
				statut: 'en_cours',
				memo: '',
				tarifId: '',
				dateNaissance: '',
				urgenceNom: '',
				urgenceTelephone: '',
				orientation: '',
				frequence: '',
				adresse: ''
			}
		});

		const dateInput = screen.getByLabelText('Date de naissance', { selector: '#date-naissance' });
		expect(dateInput).toHaveValue('');

		// Saisie au format jj/mm/aaaa
		await fireEvent.input(dateInput, { target: { value: '20/06/1899' } });
		expect(dateInput).toHaveValue('20/06/1899');
	});
});
