import { describe, expect, test, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import RdvDialog from '$lib/components/RdvDialog.svelte';
import { mockClients, mockTarifs } from './mocks';
import type { RdvCreateResult } from '$lib/types';

const mockClientsList = vi.fn();
const mockTarifsList = vi.fn();
const mockRdvCreate = vi.fn();
const mockRdvGet = vi.fn();
const mockRdvUpdate = vi.fn();

vi.mock('$lib/api', () => ({
	clientsList: () => mockClientsList(),
	tarifsList: () => mockTarifsList(),
	rdvCreate: (...args: unknown[]) => mockRdvCreate(...args),
	rdvGet: (...args: unknown[]) => mockRdvGet(...args),
	rdvUpdate: (...args: unknown[]) => mockRdvUpdate(...args)
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		error: vi.fn(),
		success: vi.fn()
	}
}));

describe('RdvDialog', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockClientsList.mockResolvedValue(mockClients);
		mockTarifsList.mockResolvedValue(mockTarifs);
	});

	test('rendu initial en mode création affiche le dialogue et charge les clients/tarifs', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		render(RdvDialog, {
			props: {
				open: true,
				presetDebut: '2026-10-15T09:00:00Z',
				presetClientId: 'client-1',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(mockClientsList).toHaveBeenCalledTimes(1);
			expect(mockTarifsList).toHaveBeenCalledTimes(1);
			expect(screen.getByText('Nouveau RDV')).toBeInTheDocument();
		});

		expect(screen.getByRole('button', { name: 'Créer' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeInTheDocument();
	});

	test('soumission de création de rendez-vous appelle rdvCreate et déclenche onSaved', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		const createResult: RdvCreateResult = {
			rdv: {
				id: 'new-rdv-1',
				client_id: 'client-1',
				client_nom: 'Marie Curie',
				tarif_id: 'tarif-1',
				tarif_nom: 'Consultation standard',
				debut: '2026-10-15T09:00:00Z',
				duree_minutes: 60,
				jitsi_url: 'https://meet.jit.si/synapt-new',
				stripe_url: null,
				stripe_id: null,
				note: null,
				statut: 'planifie',
				created_at: '2026-10-01T00:00:00Z',
				updated_at: '2026-10-01T00:00:00Z'
			},
			warnings: []
		};
		mockRdvCreate.mockResolvedValue(createResult);

		render(RdvDialog, {
			props: {
				open: true,
				presetDebut: '2026-10-15T09:00:00Z',
				presetClientId: 'client-1',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			const btn = screen.getByRole('button', { name: 'Créer' });
			expect(btn).not.toBeDisabled();
		});

		const submitBtn = screen.getByRole('button', { name: 'Créer' });
		await fireEvent.click(submitBtn);

		await waitFor(() => {
			expect(mockRdvCreate).toHaveBeenCalledTimes(1);
			expect(onSaved).toHaveBeenCalledWith(createResult);
		});
	});

	test('clic sur Annuler ferme le dialogue', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		render(RdvDialog, {
			props: {
				open: true,
				presetDebut: '2026-10-15T09:00:00Z',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Annuler' })).toBeInTheDocument();
		});

		const cancelBtn = screen.getByRole('button', { name: 'Annuler' });
		await fireEvent.click(cancelBtn);

		expect(onClose).toHaveBeenCalledTimes(1);
	});

	test('erreur de chevauchement affiche le message d erreur', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		mockRdvCreate.mockRejectedValue('Ce créneau chevauche un autre rendez-vous');

		render(RdvDialog, {
			props: {
				open: true,
				presetDebut: '2026-10-15T09:00:00Z',
				presetClientId: 'client-1',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			const btn = screen.getByRole('button', { name: 'Créer' });
			expect(btn).not.toBeDisabled();
		});

		const submitBtn = screen.getByRole('button', { name: 'Créer' });
		await fireEvent.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Ce créneau chevauche un autre rendez-vous/i)).toBeInTheDocument();
		});
		expect(onSaved).not.toHaveBeenCalled();
	});
});
