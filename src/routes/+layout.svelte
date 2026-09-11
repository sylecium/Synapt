<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/AppSidebar.svelte';

	let { children } = $props();
</script>

<ModeWatcher />
<Sidebar.SidebarProvider>
	<AppSidebar />
	<Sidebar.SidebarInset>
		<header class="flex h-10 shrink-0 items-center gap-2 border-b px-2">
			<Sidebar.Trigger />
		</header>
		<div class="relative min-h-0 flex-1 overflow-auto">
			{@render children()}
		</div>
	</Sidebar.SidebarInset>
	<Toaster />
</Sidebar.SidebarProvider>
