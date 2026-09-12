<script lang="ts">
	import { noteToEditorContent } from '$lib/notesHtml';
	import { cn } from '$lib/utils.js';

	type Props = {
		corps: string;
		class?: string;
	};

	let { corps, class: className }: Props = $props();
</script>

<div class={cn('note-editor text-sm leading-relaxed', className)}>
	{@html noteToEditorContent(corps)}
</div>
