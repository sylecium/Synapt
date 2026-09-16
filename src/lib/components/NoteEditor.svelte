<script lang="ts">
	import { onMount, untrack, type Snippet } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import { TableKit } from '@tiptap/extension-table';
	import Underline from '@tiptap/extension-underline';
	import BoldIcon from '@lucide/svelte/icons/bold';
	import ItalicIcon from '@lucide/svelte/icons/italic';
	import UnderlineIcon from '@lucide/svelte/icons/underline';
	import ListIcon from '@lucide/svelte/icons/list';
	import ListOrderedIcon from '@lucide/svelte/icons/list-ordered';
	import TableIcon from '@lucide/svelte/icons/table';
	import HeadingIcon from '@lucide/svelte/icons/heading';
	import { noteToEditorContent } from '$lib/notesHtml';
	import { cn } from '$lib/utils.js';

	type Props = {
		noteId: string;
		corps: string;
		onChange: (html: string) => void;
		leading?: Snippet;
		variant?: 'full' | 'compact';
	};

	let { noteId, corps, onChange, leading, variant = 'full' }: Props = $props();

	let host = $state<HTMLDivElement | null>(null);
	let instance: Editor | null = null;
	let emit: (html: string) => void = () => {};
	let active = $state({
		bold: false,
		italic: false,
		underline: false,
		bullet: false,
		ordered: false,
		heading: false
	});

	$effect.pre(() => {
		emit = onChange;
	});

	function syncActive(ed: Editor) {
		active = {
			bold: ed.isActive('bold'),
			italic: ed.isActive('italic'),
			underline: ed.isActive('underline'),
			bullet: ed.isActive('bulletList'),
			ordered: ed.isActive('orderedList'),
			heading: ed.isActive('heading', { level: 2 })
		};
	}

	$effect(() => {
		void noteId;
		const html = noteToEditorContent(untrack(() => corps));
		if (!instance) return;
		instance.commands.setContent(html, { emitUpdate: false });
	});

	onMount(() => {
		if (!host) return;
		const html = noteToEditorContent(corps);
		const compact = variant === 'compact';
		const ed = new Editor({
			element: host,
			extensions: [
				StarterKit.configure({
					link: { openOnClick: false }
				}),
				Underline,
				TableKit.configure({
					table: { resizable: false }
				})
			],
			content: html,
			editorProps: {
				attributes: {
					class: compact
						? 'note-editor min-h-32 px-3 py-2 text-sm leading-relaxed outline-none'
						: 'note-editor min-h-full px-8 py-5 text-[17px] leading-relaxed outline-none'
				}
			},
			onUpdate: ({ editor: next }) => {
				emit(next.getHTML());
				syncActive(next);
			},
			onSelectionUpdate: ({ editor: next }) => {
				syncActive(next);
			}
		});
		instance = ed;
		syncActive(ed);
		if (!compact) {
			queueMicrotask(() => ed.commands.focus('end'));
		}
		return () => {
			ed.destroy();
			if (instance === ed) instance = null;
		};
	});

	function run(fn: (ed: Editor) => void) {
		if (!instance) return;
		fn(instance);
		syncActive(instance);
	}

	function toolClass(on: boolean) {
		return cn(
			'inline-flex size-7 items-center justify-center rounded-md',
			on
				? 'bg-muted text-foreground'
				: 'text-muted-foreground hover:bg-muted/70 hover:text-foreground'
		);
	}
</script>

<div class={cn('flex min-h-0 flex-col', variant === 'full' && 'flex-1')}>
	<div class="flex items-center gap-1 px-2 py-1.5">
		{#if leading}
			{@render leading()}
		{/if}
		<div class="ml-auto flex items-center">
			<button
				type="button"
				class={toolClass(active.bold)}
				aria-label="Gras"
				aria-pressed={active.bold}
				onclick={() => run((ed) => ed.chain().focus().toggleBold().run())}
			>
				<BoldIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(active.italic)}
				aria-label="Italique"
				aria-pressed={active.italic}
				onclick={() => run((ed) => ed.chain().focus().toggleItalic().run())}
			>
				<ItalicIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(active.underline)}
				aria-label="Souligné"
				aria-pressed={active.underline}
				onclick={() => run((ed) => ed.chain().focus().toggleUnderline().run())}
			>
				<UnderlineIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(active.heading)}
				aria-label="Titre"
				aria-pressed={active.heading}
				onclick={() => run((ed) => ed.chain().focus().toggleHeading({ level: 2 }).run())}
			>
				<HeadingIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(active.bullet)}
				aria-label="Liste"
				aria-pressed={active.bullet}
				onclick={() => run((ed) => ed.chain().focus().toggleBulletList().run())}
			>
				<ListIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(active.ordered)}
				aria-label="Liste numérotée"
				aria-pressed={active.ordered}
				onclick={() => run((ed) => ed.chain().focus().toggleOrderedList().run())}
			>
				<ListOrderedIcon class="size-3.5" />
			</button>
			<button
				type="button"
				class={toolClass(false)}
				aria-label="Insérer un tableau"
				onclick={() =>
					run((ed) =>
						ed.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
					)}
			>
				<TableIcon class="size-3.5" />
			</button>
		</div>
	</div>

	<div
		class={cn(
			'min-h-0 overflow-y-auto',
			variant === 'full' ? 'flex-1' : 'max-h-64 rounded-md border'
		)}
	>
		<div bind:this={host} class={variant === 'full' ? 'h-full' : ''}></div>
	</div>
</div>
