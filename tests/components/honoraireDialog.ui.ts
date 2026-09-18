import { describe, expect, test, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import HonoraireDialog from '$lib/components/HonoraireDialog.svelte';
import { mockClients, mockHonoraireDetail, mockRdvs } from './mocks';

const mockClientsList = vi.fn();
const mockHonorairesRdvsDisponibles = vi.fn();
const mockHonorairesCreate = vi.fn();
const mockHonorairesOuvrir = vi.fn();
const mockOpenPath = vi.fn();

vi.mock('$lib/api', () => ({
	clientsList: () => mockClientsList(),
	honorairesRdvsDisponibles: (id: string) => mockHonorairesRdvsDisponibles(id),
	honorairesCreate: (...args: unknown[]) => mockHonorairesCreate(...args),
	honorairesOuvrir: (...args: unknown[]) => mockHonorairesOuvrir(...args)
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
	openPath: (...args: unknown[]) => mockOpenPath(...args)
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		error: vi.fn(),
		success: vi.fn()
	}
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('HonoraireDialog', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockClientsList.mockResolvedValue(mockClients);
		mockHonorairesRdvsDisponibles.mockResolvedValue(mockRdvs);
		mockHonorairesCreate.mockResolvedValue(mockHonoraireDetail);
		mockHonorairesOuvrir.mockResolvedValue('/home/user/Synapt/honoraires/NH-2026-001.pdf');
		mockOpenPath.mockResolvedValue(undefined);
	});

	test('rendu initial et chargement des séances disponibles', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		render(HonoraireDialog, {
			props: {
				open: true,
				clientId: 'client-1',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(mockHonorairesRdvsDisponibles).toHaveBeenCalledWith('client-1');
			expect(screen.getByText("Nouvelle note d'honoraires")).toBeInTheDocument();
		});

		expect(screen.getByRole('button', { name: 'Générer' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Annuler/i })).toBeInTheDocument();
	});

	test('sélection des séances et soumission avec honorairesCreate', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		render(HonoraireDialog, {
			props: {
				open: true,
				clientId: 'client-1',
				rdvIds: ['rdv-1'],
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(mockHonorairesRdvsDisponibles).toHaveBeenCalledWith('client-1');
			const submitBtn = screen.getByRole('button', { name: 'Générer' });
			expect(submitBtn).not.toBeDisabled();
		});

		const submitBtn = screen.getByRole('button', { name: 'Générer' });
		await fireEvent.click(submitBtn);

		await waitFor(() => {
			expect(mockHonorairesCreate).toHaveBeenCalledTimes(1);
			expect(mockHonorairesCreate).toHaveBeenCalledWith({
				rdv_ids: ['rdv-1'],
				moyen_paiement: 'especes'
			});
			expect(mockHonorairesOuvrir).toHaveBeenCalledWith('hon-1');
			expect(mockOpenPath).toHaveBeenCalledWith('/home/user/Synapt/honoraires/NH-2026-001.pdf');
			expect(onSaved).toHaveBeenCalledWith(mockHonoraireDetail);
		});
	});

	test('bouton générer désactivé si aucune séance n est sélectionnée', async () => {
		const onClose = vi.fn();
		const onSaved = vi.fn();

		// Deux RDVs disponibles et aucun pré-sélectionné
		render(HonoraireDialog, {
			props: {
				open: true,
				clientId: 'client-1',
				rdvIds: [],
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(mockHonorairesRdvsDisponibles).toHaveBeenCalledWith('client-1');
		});

		const submitBtn = screen.getByRole('button', { name: 'Générer' });
		expect(submitBtn).toBeDisabled();
	});

	test('affiche un message quand aucune séance n est facturable', async () => {
		mockHonorairesRdvsDisponibles.mockResolvedValue([]);
		const onClose = vi.fn();
		const onSaved = vi.fn();

		render(HonoraireDialog, {
			props: {
				open: true,
				clientId: 'client-1',
				onClose,
				onSaved
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Aucune séance disponible pour ce client.')).toBeInTheDocument();
		});

		const submitBtn = screen.getByRole('button', { name: 'Générer' });
		expect(submitBtn).toBeDisabled();
	});
});
