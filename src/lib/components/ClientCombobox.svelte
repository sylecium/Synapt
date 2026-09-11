<script lang="ts">
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import { tick } from 'svelte';
	import type { Client } from '$lib/types';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Command from '$lib/components/ui/command/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';

	type Props = {
		clients: Client[];
		value: string;
		loaded?: boolean;
		onValueChange: (id: string) => void;
	};

	let { clients, value, loaded = true, onValueChange }: Props = $props();

	let open = $state(false);
	let triggerRef = $state<HTMLButtonElement>(null!);

	const selectedLabel = $derived(clients.find((c) => c.id === value)?.nom);

	function closeAndFocusTrigger() {
		open = false;
		tick().then(() => {
			triggerRef.focus();
		});
	}
</script>

{#if !loaded}
	<Button variant="outline" class="w-full justify-between" disabled>Chargement…</Button>
{:else if clients.length === 0}
	<a href="/clients" class="text-primary text-sm underline-offset-4 hover:underline">Créer un client</a>
{:else}
	<Popover.Root bind:open>
		<Popover.Trigger bind:ref={triggerRef}>
			{#snippet child({ props })}
				<Button
					variant="outline"
					class="w-full justify-between"
					{...props}
					role="combobox"
					aria-expanded={open}
				>
					{selectedLabel ?? 'Choisir un client'}
					<ChevronsUpDownIcon class="ms-2 size-4 shrink-0 opacity-50" />
				</Button>
			{/snippet}
		</Popover.Trigger>
		<Popover.Content class="w-[var(--bits-popover-anchor-width)] p-0">
			<Command.Root>
				<Command.Input placeholder="Rechercher un client..." />
				<Command.List>
					<Command.Empty>Aucun client trouvé.</Command.Empty>
					<Command.Group>
						{#each clients as client (client.id)}
							<Command.Item
								value={client.nom}
								onSelect={() => {
									onValueChange(client.id);
									closeAndFocusTrigger();
								}}
							>
								{client.nom}
							</Command.Item>
						{/each}
					</Command.Group>
				</Command.List>
			</Command.Root>
		</Popover.Content>
	</Popover.Root>
{/if}
