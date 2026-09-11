<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { ntfyTest, settingsGet, settingsSet } from '$lib/api';
	import type { SettingsPublic } from '$lib/types';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';

	let loaded = $state<SettingsPublic | null>(null);
	let ntfyServeur = $state('');
	let ntfyTopic = $state('');
	let ntfyToken = $state('');
	let rappel24h = $state(true);
	let rappel1h = $state(true);
	let stripeSecretKey = $state('');
	let saving = $state(false);
	let testing = $state(false);

	onMount(async () => {
		loaded = await settingsGet();
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
				ntfy_serveur: ntfyServeur,
				ntfy_topic: ntfyTopic,
				ntfy_token: ntfyToken,
				rappel_24h: rappel24h,
				rappel_1h: rappel1h,
				stripe_secret_key: stripeSecretKey
			});
			loaded = await settingsGet();
			ntfyToken = '';
			stripeSecretKey = '';
			toast.success('Réglages enregistrés');
		} catch (e) {
			toast.error(String(e));
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
			toast.error(String(e));
		} finally {
			testing = false;
		}
	}
</script>

<div class="flex flex-col gap-4 p-4">
	<PageHeader title="Réglages" />

	{#if !loaded}
		<Skeleton class="h-64 max-w-lg" />
	{:else}
		<section class="flex max-w-lg flex-col gap-4">
			<h2 class="text-sm font-medium">ntfy</h2>
			<div class="flex flex-col gap-2">
				<Label for="ntfy-serveur">Serveur</Label>
				<Input id="ntfy-serveur" bind:value={ntfyServeur} />
			</div>
			<div class="flex flex-col gap-2">
				<Label for="ntfy-topic">Topic</Label>
				<Input id="ntfy-topic" bind:value={ntfyTopic} />
				<p class="text-muted-foreground text-xs">
					Nom secret du canal. Sans topic, les rappels téléphone sont coupés.
				</p>
			</div>
			<div class="flex flex-col gap-2">
				<Label for="ntfy-token">Token</Label>
				<Input
					id="ntfy-token"
					type="password"
					bind:value={ntfyToken}
					placeholder={secretPlaceholder(loaded.ntfy_token_configured, loaded.ntfy_token_last4)}
				/>
				<p class="text-muted-foreground text-xs">Optionnel, si le serveur ntfy l'exige.</p>
			</div>
			<div class="flex items-center gap-2">
				<Switch id="rappel-24h" bind:checked={rappel24h} />
				<Label for="rappel-24h">Rappel 24 h</Label>
			</div>
			<div class="flex items-center gap-2">
				<Switch id="rappel-1h" bind:checked={rappel1h} />
				<Label for="rappel-1h">Rappel 1 h</Label>
			</div>
			<Button variant="outline" onclick={testNtfy} disabled={testing}>Tester</Button>
		</section>

		<section class="flex max-w-lg flex-col gap-4">
			<h2 class="text-sm font-medium">Stripe</h2>
			<div class="flex flex-col gap-2">
				<Label for="stripe-key">Clé secrète</Label>
				<Input
					id="stripe-key"
					type="password"
					bind:value={stripeSecretKey}
					placeholder={secretPlaceholder(loaded.stripe_configured, loaded.stripe_last4)}
				/>
				<p class="text-muted-foreground text-xs">
					Collez une clé secrète sk_…. Elle reste sur cette machine.
				</p>
			</div>
		</section>

		<Button onclick={save} disabled={saving}>Enregistrer</Button>
	{/if}
</div>
