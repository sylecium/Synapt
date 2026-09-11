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
		{@render children()}
	</Sidebar.SidebarInset>
	<Toaster />
</Sidebar.SidebarProvider>
