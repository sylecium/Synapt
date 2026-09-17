<script lang="ts">
	import { page } from '$app/state';
	import { toggleMode } from 'mode-watcher';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import UsersIcon from '@lucide/svelte/icons/users';
	import ReceiptIcon from '@lucide/svelte/icons/receipt';
	import TagIcon from '@lucide/svelte/icons/tag';
	import StickyNoteIcon from '@lucide/svelte/icons/sticky-note';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	const items = [
		{ title: 'Tableau de bord', href: '/', icon: LayoutDashboardIcon },
		{ title: 'Agenda', href: '/agenda', icon: CalendarIcon },
		{ title: 'Clients', href: '/clients', icon: UsersIcon },
		{ title: 'Honoraires', href: '/honoraires', icon: ReceiptIcon },
		{ title: 'Tarifs', href: '/tarifs', icon: TagIcon },
		{ title: 'Notes', href: '/notes', icon: StickyNoteIcon },
		{ title: 'Réglages', href: '/reglages', icon: SettingsIcon }
	];

	function isActive(href: string): boolean {
		const pathname = page.url.pathname;
		if (href === '/') return pathname === '/';
		return pathname === href || pathname.startsWith(`${href}/`);
	}
</script>

<Sidebar.Root collapsible="icon">
	<Sidebar.Header>
		<div class="px-2 py-1 text-sm font-semibold">Synapt</div>
	</Sidebar.Header>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each items as item (item.href)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton isActive={isActive(item.href)}>
								{#snippet child({ props })}
									<a href={item.href} {...props}>
										<item.icon />
										<span>{item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={toggleMode} tooltipContent="Clair / sombre">
					<SunIcon class="dark:hidden" />
					<MoonIcon class="hidden dark:block" />
					<span>Clair / sombre</span>
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
</Sidebar.Root>
