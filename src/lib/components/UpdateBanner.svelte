<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { relaunch } from '@tauri-apps/plugin-process';
	import type { Update } from '@tauri-apps/plugin-updater';
	import { Button } from '$lib/components/ui/button/index.js';
	import { installUpdateMessage } from '$lib/updater-errors';
	import { probeUpdate, UPDATE_CHECK_INTERVAL_MS } from '$lib/updater';

	let update = $state<Update | null>(null);
	let dismissed = $state(false);
	let installing = $state(false);
	let percent = $state<number | null>(null);

	async function refresh() {
		if (dismissed || installing || update) return;
		update = await probeUpdate();
	}

	onMount(() => {
		void refresh();
		const id = setInterval(() => void refresh(), UPDATE_CHECK_INTERVAL_MS);
		return () => clearInterval(id);
	});

	async function install() {
		if (!update || installing) return;
		installing = true;
		try {
			let contentLength = 0;
			let downloaded = 0;
			await update.downloadAndInstall((event) => {
				switch (event.event) {
					case 'Started':
						contentLength = event.data.contentLength ?? 0;
						downloaded = 0;
						percent = contentLength > 0 ? 0 : null;
						break;
					case 'Progress':
						downloaded += event.data.chunkLength;
						if (contentLength > 0) {
							percent = Math.min(100, Math.round((downloaded / contentLength) * 100));
						}
						break;
					case 'Finished':
						percent = 100;
						break;
					default: {
						const _exhaustive: never = event;
						void _exhaustive;
					}
				}
			});
			await relaunch();
		} catch (e) {
			toast.error(installUpdateMessage(e));
			installing = false;
		}
	}
</script>

{#if update && !dismissed}
	<div class="flex flex-wrap items-center gap-2 border-b bg-accent px-3 py-2 text-sm">
		<p class="min-w-0 flex-1">
			Version {update.version} disponible.
			{#if installing && percent !== null}
				<span class="font-mono tabular-nums opacity-80">{percent}%</span>
			{/if}
		</p>
		<Button size="sm" disabled={installing} onclick={() => void install()}>Installer</Button>
		<Button size="sm" variant="ghost" disabled={installing} onclick={() => (dismissed = true)}>
			Plus tard
		</Button>
	</div>
{/if}
