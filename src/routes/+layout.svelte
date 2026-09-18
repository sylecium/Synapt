<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import UpdateBanner from '$lib/components/UpdateBanner.svelte';
	import { flushAllDebouncedNotes } from '$lib/debouncedNoteSave';

	let { children } = $props();

	onMount(() => {
		let unlisten: (() => void) | undefined;
		(async () => {
			try {
				const { getCurrentWindow } = await import('@tauri-apps/api/window');
				const win = getCurrentWindow();
				unlisten = await win.onCloseRequested(async () => {
					await Promise.race([
						flushAllDebouncedNotes(),
						new Promise((resolve) => setTimeout(resolve, 1500))
					]);
				});
			} catch {
				// Environnement hors Tauri (tests, ssr)
			}
		})();
		return () => {
			unlisten?.();
		};
	});
</script>

<ModeWatcher />
<Sidebar.SidebarProvider>
	<AppSidebar />
	<Sidebar.SidebarInset>
		<header class="flex h-10 shrink-0 items-center gap-2 border-b px-2">
			<Sidebar.Trigger />
		</header>
		<UpdateBanner />
		<div class="relative min-h-0 flex-1 overflow-auto">
			{@render children()}
		</div>
	</Sidebar.SidebarInset>
	<Toaster />
</Sidebar.SidebarProvider>
