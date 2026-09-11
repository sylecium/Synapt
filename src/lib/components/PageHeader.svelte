<script lang="ts">
	import type { Snippet } from 'svelte';
	type Props = { title: string; children?: Snippet };
	let { title, children }: Props = $props();
</script>

<div class="flex items-center justify-between gap-4">
	<h1 class="text-lg font-semibold tracking-tight">{title}</h1>
	{#if children}
		<div class="flex items-center gap-2">{@render children()}</div>
	{/if}
</div>
