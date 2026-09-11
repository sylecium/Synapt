<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';

	type Props = {
		title: string;
		description: string;
		href?: string;
		actionLabel?: string;
		onclick?: () => void;
	};
	let { title, description, href, actionLabel, onclick }: Props = $props();
</script>

<div class="rounded-md border bg-card px-4 py-6">
	<p class="text-sm font-medium">{title}</p>
	<p class="text-muted-foreground mt-1 text-sm">{description}</p>
	{#if actionLabel && (href || onclick)}
		<Button class="mt-4" size="sm" {href} {onclick}>{actionLabel}</Button>
	{/if}
</div>
