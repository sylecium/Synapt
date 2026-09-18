<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import BuildingIcon from '@lucide/svelte/icons/building';
	import BellIcon from '@lucide/svelte/icons/bell';
	import CreditCardIcon from '@lucide/svelte/icons/credit-card';
	import SendIcon from '@lucide/svelte/icons/send';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { ntfyTest, settingsGet, settingsSet } from '$lib/api';
	import { userMessage } from '$lib/errors';
	import type { SettingsPublic } from '$lib/types';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	const mentionTvaDefaut = 'TVA au taux normal de 20 % (CGI, art. 278)';

	let loaded = $state<SettingsPublic | null>(null);
	let cabinetNom = $state('');
	let cabinetAdresse = $state('');
	let cabinetTelephone = $state('');
	let cabinetEmail = $state('');
	let cabinetSiret = $state('');
	let mentionTva = $state('');
	let prefixeNumero = $state('');
	let ntfyServeur = $state('');
	let ntfyTopic = $state('');
	let ntfyToken = $state('');
	let rappel24h = $state(true);
	let rappel1h = $state(true);
	let stripeSecretKey = $state('');
	let ntfyTokenClear = $state(false);
	let stripeSecretClear = $state(false);
	let saving = $state(false);
	let testing = $state(false);

	onMount(async () => {
		loaded = await settingsGet();
		cabinetNom = loaded.cabinet_nom;
		cabinetAdresse = loaded.cabinet_adresse;
		cabinetTelephone = loaded.cabinet_telephone;
		cabinetEmail = loaded.cabinet_email;
		cabinetSiret = loaded.cabinet_siret;
		mentionTva = loaded.mention_tva.trim() || mentionTvaDefaut;
		prefixeNumero = loaded.prefixe_numero;
		ntfyServeur = loaded.ntfy_serveur;
		ntfyTopic = loaded.ntfy_topic;
		rappel24h = loaded.rappel_24h;
		rappel1h = loaded.rappel_1h;
	});

	function secretPlaceholder(configured: boolean, last4: string): string {
		return configured ? `••••${last4}` : '';
	}

	async function save() {
		saving = true;
		try {
			await settingsSet({
				cabinet_nom: cabinetNom,
				cabinet_adresse: cabinetAdresse,
				cabinet_telephone: cabinetTelephone,
				cabinet_email: cabinetEmail,
				cabinet_siret: cabinetSiret,
				mention_tva: mentionTva,
				prefixe_numero: prefixeNumero,
				ntfy_serveur: ntfyServeur,
				ntfy_topic: ntfyTopic,
				ntfy_token: ntfyToken,
				ntfy_token_clear: ntfyTokenClear,
				rappel_24h: rappel24h,
				rappel_1h: rappel1h,
				stripe_secret_key: stripeSecretKey,
				stripe_secret_clear: stripeSecretClear
			});
			loaded = await settingsGet();
			ntfyToken = '';
			stripeSecretKey = '';
			ntfyTokenClear = false;
			stripeSecretClear = false;
			toast.success('Réglages enregistrés');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			saving = false;
		}
	}

	async function testNtfy() {
		testing = true;
		try {
			await ntfyTest();
			toast.success('Notification de test envoyée');
		} catch (e) {
			toast.error(userMessage(e));
		} finally {
			testing = false;
		}
	}
</script>

<div class="flex flex-col gap-6 p-4 max-w-3xl">
	<PageHeader title="Réglages" />

	{#if !loaded}
		<div class="flex flex-col gap-6">
			<Skeleton class="h-64 w-full rounded-xl" />
			<Skeleton class="h-64 w-full rounded-xl" />
			<Skeleton class="h-40 w-full rounded-xl" />
		</div>
	{:else}
		<!-- Section Cabinet -->
		<section class="flex flex-col gap-5 rounded-xl border bg-card p-5 sm:p-6 shadow-xs">
			<div class="flex items-start justify-between gap-4">
				<div class="flex items-center gap-3">
					<div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
						<BuildingIcon class="size-4" />
					</div>
					<div>
						<h2 class="text-base font-semibold tracking-tight text-foreground">Cabinet</h2>
						<p class="text-xs text-muted-foreground">
							Coordonnées professionnelles et mentions légales pour vos notes d'honoraires.
						</p>
					</div>
				</div>
			</div>

			<Separator />

			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<Label for="cab-nom">Nom du cabinet ou praticien</Label>
					<Input id="cab-nom" bind:value={cabinetNom} placeholder="Ex: Dr. Jane Doe" />
				</div>

				<div class="flex flex-col gap-2">
					<Label for="cab-adresse">Adresse professionnelle</Label>
					<Textarea
						id="cab-adresse"
						bind:value={cabinetAdresse}
						rows={3}
						placeholder="Numéro, voie, code postal et ville"
					/>
				</div>

				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="flex flex-col gap-2">
						<Label for="cab-telephone">Téléphone</Label>
						<Input id="cab-telephone" type="tel" bind:value={cabinetTelephone} placeholder="06 00 00 00 00" />
					</div>
					<div class="flex flex-col gap-2">
						<Label for="cab-email">Email</Label>
						<Input id="cab-email" type="email" bind:value={cabinetEmail} placeholder="contact@cabinet.fr" />
					</div>
				</div>

				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="flex flex-col gap-2">
						<Label for="cab-siret">Numéro SIRET</Label>
						<Input id="cab-siret" bind:value={cabinetSiret} placeholder="14 chiffres" />
					</div>
					<div class="flex flex-col gap-2">
						<Label for="cab-prefixe">Préfixe de numérotation</Label>
						<Input id="cab-prefixe" bind:value={prefixeNumero} placeholder="Ex: NH" />
					</div>
				</div>
				<p class="-mt-2 text-xs text-muted-foreground">
					Préfixe optionnel pour les numéros de notes d'honoraires. Exemple NH - 2026-0001 (vide = 2026-0001).
				</p>

				<div class="flex flex-col gap-2">
					<Label for="cab-mention-tva">Mention légale TVA</Label>
					<Input id="cab-mention-tva" bind:value={mentionTva} />
					<p class="text-xs text-muted-foreground">
						Mention obligatoire affichée en pied de note. Défaut : {mentionTvaDefaut}.
					</p>
				</div>
			</div>
		</section>

		<!-- Section ntfy -->
		<section class="flex flex-col gap-5 rounded-xl border bg-card p-5 sm:p-6 shadow-xs">
			<div class="flex items-start justify-between gap-4">
				<div class="flex items-center gap-3">
					<div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
						<BellIcon class="size-4" />
					</div>
					<div>
						<h2 class="text-base font-semibold tracking-tight text-foreground">Rappels ntfy</h2>
						<p class="text-xs text-muted-foreground">
							Notifications instantanées et rappels automatiques des consultations sur votre téléphone.
						</p>
					</div>
				</div>
				{#if ntfyTopic.trim()}
					<Badge variant="secondary">Actif</Badge>
				{:else}
					<Badge variant="outline" class="text-muted-foreground">Désactivé</Badge>
				{/if}
			</div>

			<Separator />

			<div class="flex flex-col gap-4">
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="flex flex-col gap-2">
						<Label for="ntfy-serveur">Serveur ntfy</Label>
						<Input id="ntfy-serveur" bind:value={ntfyServeur} placeholder="https://ntfy.sh" />
					</div>
					<div class="flex flex-col gap-2">
						<Label for="ntfy-topic">Topic secret</Label>
						<Input id="ntfy-topic" bind:value={ntfyTopic} placeholder="mon-cabinet-secret-..." />
					</div>
				</div>
				<p class="-mt-2 text-xs text-muted-foreground">
					Nom secret de votre canal de notification. Sans topic, les rappels sont suspendus.
				</p>

				<div class="flex flex-col gap-2">
					<Label for="ntfy-token">Token d'accès (optionnel)</Label>
					<Input
						id="ntfy-token"
						type="password"
						bind:value={ntfyToken}
						placeholder={secretPlaceholder(loaded.ntfy_token_configured, loaded.ntfy_token_last4)}
					/>
					<p class="text-xs text-muted-foreground">Requis uniquement si votre serveur ntfy exige une authentification.</p>
					{#if loaded.ntfy_token_configured}
						<label class="flex items-center gap-2 pt-1 text-sm">
							<input type="checkbox" bind:checked={ntfyTokenClear} class="rounded border-input text-primary" />
							<span>Effacer le token actuellement enregistré</span>
						</label>
					{/if}
				</div>

				<div class="flex flex-col gap-3 rounded-lg border bg-muted/30 p-3.5">
					<div class="flex items-center justify-between gap-4">
						<div class="flex flex-col">
							<Label for="rappel-24h" class="cursor-pointer font-medium text-foreground">Rappel 24 h avant</Label>
							<span class="text-xs text-muted-foreground">Notification envoyée la veille de la consultation</span>
						</div>
						<Switch id="rappel-24h" bind:checked={rappel24h} />
					</div>
					<Separator />
					<div class="flex items-center justify-between gap-4">
						<div class="flex flex-col">
							<Label for="rappel-1h" class="cursor-pointer font-medium text-foreground">Rappel 1 h avant</Label>
							<span class="text-xs text-muted-foreground">Notification envoyée une heure avant la consultation</span>
						</div>
						<Switch id="rappel-1h" bind:checked={rappel1h} />
					</div>
				</div>

				<div class="flex items-center justify-between pt-1">
					<p class="text-xs text-muted-foreground">Testez l'envoi vers l'application ntfy de votre téléphone.</p>
					<Button variant="outline" size="sm" onclick={testNtfy} disabled={testing}>
						<SendIcon class="mr-1.5 size-3.5" />
						{testing ? 'Envoi en cours...' : 'Tester la notification'}
					</Button>
				</div>
			</div>
		</section>

		<!-- Section Stripe -->
		<section class="flex flex-col gap-5 rounded-xl border bg-card p-5 sm:p-6 shadow-xs">
			<div class="flex items-start justify-between gap-4">
				<div class="flex items-center gap-3">
					<div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
						<CreditCardIcon class="size-4" />
					</div>
					<div>
						<h2 class="text-base font-semibold tracking-tight text-foreground">Paiements Stripe</h2>
						<p class="text-xs text-muted-foreground">
							Génération automatique des liens de paiement sécurisés pour les téléconsultations.
						</p>
					</div>
				</div>
				{#if loaded.stripe_configured}
					<Badge variant="secondary">Configuré</Badge>
				{:else}
					<Badge variant="outline" class="text-muted-foreground">Non configuré</Badge>
				{/if}
			</div>

			<Separator />

			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<Label for="stripe-key">Clé secrète API</Label>
					<Input
						id="stripe-key"
						type="password"
						bind:value={stripeSecretKey}
						placeholder={secretPlaceholder(loaded.stripe_configured, loaded.stripe_last4)}
					/>
					<p class="text-xs text-muted-foreground">
						Clé secrète commençant par sk_... stockée localement sur ce poste.
					</p>
					{#if loaded.stripe_configured}
						<label class="flex items-center gap-2 pt-1 text-sm">
							<input type="checkbox" bind:checked={stripeSecretClear} class="rounded border-input text-primary" />
							<span>Effacer la clé secrète actuellement enregistrée</span>
						</label>
					{/if}
				</div>
			</div>
		</section>

		<!-- Action Enregistrer -->
		<div class="flex items-center justify-end pt-2 pb-8">
			<Button onclick={save} disabled={saving} class="min-w-40">
				<CheckIcon class="mr-1.5 size-4" />
				{saving ? 'Enregistrement...' : 'Enregistrer les réglages'}
			</Button>
		</div>
	{/if}
</div>
