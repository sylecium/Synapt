<script lang="ts">
	import { onMount, untrack, type Snippet } from 'svelte';
	import { Editor, Extension } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import { TableKit } from '@tiptap/extension-table';
	import BoldIcon from '@lucide/svelte/icons/bold';
	import ItalicIcon from '@lucide/svelte/icons/italic';
	import UnderlineIcon from '@lucide/svelte/icons/underline';
	import ListIcon from '@lucide/svelte/icons/list';
	import ListOrderedIcon from '@lucide/svelte/icons/list-ordered';
	import TableIcon from '@lucide/svelte/icons/table';
	import HeadingIcon from '@lucide/svelte/icons/heading';
	import Rows2Icon from '@lucide/svelte/icons/rows-2';
	import Columns2Icon from '@lucide/svelte/icons/columns-2';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
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
		heading: false,
		table: false
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
			heading: ed.isActive('heading', { level: 2 }),
			table: ed.isActive('table')
		};
	}

	$effect(() => {
		void noteId;
		const html = noteToEditorContent(untrack(() => corps));
		if (!instance) return;
		instance.commands.setContent(html, { emitUpdate: false });
	});

	const CustomShortcuts = Extension.create({
		name: 'customShortcuts',
		addKeyboardShortcuts() {
			return {
				'Mod-b': () => this.editor.commands.toggleBold(),
				'Mod-B': () => this.editor.commands.toggleBold(),
				'Mod-i': () => this.editor.commands.toggleItalic(),
				'Mod-I': () => this.editor.commands.toggleItalic(),
				'Mod-u': () => this.editor.commands.toggleUnderline(),
				'Mod-U': () => this.editor.commands.toggleUnderline(),
				'Mod-Alt-2': () => this.editor.commands.toggleHeading({ level: 2 }),
				'Mod-Shift-h': () => this.editor.commands.toggleHeading({ level: 2 }),
				'Mod-Shift-H': () => this.editor.commands.toggleHeading({ level: 2 }),
				'Mod-Shift-8': () => this.editor.commands.toggleBulletList(),
				'Mod-Shift-l': () => this.editor.commands.toggleBulletList(),
				'Mod-Shift-L': () => this.editor.commands.toggleBulletList(),
				'Mod-Shift-7': () => this.editor.commands.toggleOrderedList(),
				'Mod-Shift-o': () => this.editor.commands.toggleOrderedList(),
				'Mod-Shift-O': () => this.editor.commands.toggleOrderedList(),
				'Mod-Alt-t': () => {
					if (!this.editor.isActive('table')) {
						return this.editor.commands.insertTable({ rows: 3, cols: 3, withHeaderRow: true });
					}
					return false;
				},
				'Mod-Alt-T': () => {
					if (!this.editor.isActive('table')) {
						return this.editor.commands.insertTable({ rows: 3, cols: 3, withHeaderRow: true });
					}
					return false;
				},
				'Mod-Alt-ArrowDown': () => this.editor.commands.addRowAfter(),
				'Mod-Alt-ArrowUp': () => this.editor.commands.addRowBefore(),
				'Mod-Alt-ArrowRight': () => this.editor.commands.addColumnAfter(),
				'Mod-Alt-ArrowLeft': () => this.editor.commands.addColumnBefore(),
				'Mod-Alt-Down': () => this.editor.commands.addRowAfter(),
				'Mod-Alt-Up': () => this.editor.commands.addRowBefore(),
				'Mod-Alt-Right': () => this.editor.commands.addColumnAfter(),
				'Mod-Alt-Left': () => this.editor.commands.addColumnBefore()
			};
		}
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
				TableKit.configure({
					table: {
						resizable: true,
						handleWidth: 5,
						cellMinWidth: 40,
						lastColumnResizable: true
					}
				}),
				CustomShortcuts
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

	function insertDefaultTable() {
		run((ed) => ed.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run());
	}

	function addRowAfter() {
		run((ed) => ed.chain().focus().addRowAfter().run());
	}

	function addRowBefore() {
		run((ed) => ed.chain().focus().addRowBefore().run());
	}

	function deleteRow() {
		run((ed) => ed.chain().focus().deleteRow().run());
	}

	function addColAfter() {
		run((ed) => ed.chain().focus().addColumnAfter().run());
	}

	function addColBefore() {
		run((ed) => ed.chain().focus().addColumnBefore().run());
	}

	function deleteCol() {
		run((ed) => ed.chain().focus().deleteColumn().run());
	}

	function deleteTable() {
		run((ed) => ed.chain().focus().deleteTable().run());
	}

	function handleContextMenu(e: MouseEvent) {
		if (!instance) return;
		const pos = instance.view.posAtCoords({ left: e.clientX, top: e.clientY });
		if (pos) {
			const sel = instance.state.selection;
			if (sel.empty || pos.pos < sel.from || pos.pos > sel.to) {
				instance.commands.setTextSelection(pos.pos);
			}
			syncActive(instance);
		}
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

{#snippet toolButton(label: string, shortcut: string, isOn: boolean, onclick: () => void, Icon: any)}
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<button
					{...props}
					type="button"
					class={toolClass(isOn)}
					aria-label="{label} ({shortcut})"
					aria-pressed={isOn}
					{onclick}
				>
					<Icon class="size-3.5" />
				</button>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Content side="bottom" class="flex items-center gap-1.5 px-2 py-1 text-xs">
			<span>{label}</span>
			<kbd class="rounded bg-muted/80 px-1 py-0.5 font-mono text-[10px] text-muted-foreground">{shortcut}</kbd>
		</Tooltip.Content>
	</Tooltip.Root>
{/snippet}

<div class={cn('flex min-h-0 flex-col', variant === 'full' && 'flex-1')}>
	<div class="flex items-center gap-1 px-2 py-1.5">
		{#if leading}
			{@render leading()}
		{/if}
		<Tooltip.Provider delayDuration={250}>
			<div class="ml-auto flex items-center gap-0.5">
				{@render toolButton('Gras', 'Ctrl+B', active.bold, () => run((ed) => ed.chain().focus().toggleBold().run()), BoldIcon)}
				{@render toolButton('Italique', 'Ctrl+I', active.italic, () => run((ed) => ed.chain().focus().toggleItalic().run()), ItalicIcon)}
				{@render toolButton('Souligné', 'Ctrl+U', active.underline, () => run((ed) => ed.chain().focus().toggleUnderline().run()), UnderlineIcon)}
				{@render toolButton('Titre', 'Ctrl+Alt+2', active.heading, () => run((ed) => ed.chain().focus().toggleHeading({ level: 2 }).run()), HeadingIcon)}
				{@render toolButton('Liste à puces', 'Ctrl+Shift+8', active.bullet, () => run((ed) => ed.chain().focus().toggleBulletList().run()), ListIcon)}
				{@render toolButton('Liste numérotée', 'Ctrl+Shift+7', active.ordered, () => run((ed) => ed.chain().focus().toggleOrderedList().run()), ListOrderedIcon)}

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<button
								{...props}
								type="button"
								class={toolClass(active.table)}
								aria-label={active.table ? 'Tableau actif' : 'Insérer un tableau (Ctrl+Alt+T)'}
								aria-pressed={active.table}
								onclick={() => {
									if (!active.table) insertDefaultTable();
								}}
							>
								<TableIcon class="size-3.5" />
							</button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom" class="flex items-center gap-1.5 px-2 py-1 text-xs">
						<span>{active.table ? 'Tableau actif' : 'Insérer un tableau'}</span>
						<kbd class="rounded bg-muted/80 px-1 py-0.5 font-mono text-[10px] text-muted-foreground">Ctrl+Alt+T</kbd>
					</Tooltip.Content>
				</Tooltip.Root>

				{#if active.table}
					<div class="mx-1 h-3.5 w-px bg-border"></div>

					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={toolClass(false)}
									aria-label="Ajouter une ligne en dessous (Ctrl+Alt+↓)"
									onclick={addRowAfter}
								>
									<Rows2Icon class="size-3.5" />
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content side="bottom" class="flex items-center gap-1.5 px-2 py-1 text-xs">
							<span>+ Ligne en dessous</span>
							<kbd class="rounded bg-muted/80 px-1 py-0.5 font-mono text-[10px] text-muted-foreground">Ctrl+Alt+↓</kbd>
						</Tooltip.Content>
					</Tooltip.Root>

					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={toolClass(false)}
									aria-label="Ajouter une colonne à droite (Ctrl+Alt+→)"
									onclick={addColAfter}
								>
									<Columns2Icon class="size-3.5" />
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content side="bottom" class="flex items-center gap-1.5 px-2 py-1 text-xs">
							<span>+ Colonne à droite</span>
							<kbd class="rounded bg-muted/80 px-1 py-0.5 font-mono text-[10px] text-muted-foreground">Ctrl+Alt+→</kbd>
						</Tooltip.Content>
					</Tooltip.Root>

					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={toolClass(false)}
									aria-label="Options du tableau"
								>
									<EllipsisIcon class="size-3.5" />
								</button>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content align="end" class="w-56 text-xs">
							<DropdownMenu.Item onSelect={addRowAfter}>
								<span>Ajouter une ligne en dessous</span>
								<DropdownMenu.Shortcut>Ctrl+Alt+↓</DropdownMenu.Shortcut>
							</DropdownMenu.Item>
							<DropdownMenu.Item onSelect={addRowBefore}>
								<span>Ajouter une ligne au-dessus</span>
								<DropdownMenu.Shortcut>Ctrl+Alt+↑</DropdownMenu.Shortcut>
							</DropdownMenu.Item>
							<DropdownMenu.Item onSelect={deleteRow}>
								<span>Supprimer la ligne</span>
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item onSelect={addColAfter}>
								<span>Ajouter une colonne à droite</span>
								<DropdownMenu.Shortcut>Ctrl+Alt+→</DropdownMenu.Shortcut>
							</DropdownMenu.Item>
							<DropdownMenu.Item onSelect={addColBefore}>
								<span>Ajouter une colonne à gauche</span>
								<DropdownMenu.Shortcut>Ctrl+Alt+←</DropdownMenu.Shortcut>
							</DropdownMenu.Item>
							<DropdownMenu.Item onSelect={deleteCol}>
								<span>Supprimer la colonne</span>
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item variant="destructive" onSelect={deleteTable}>
								<span>Supprimer le tableau</span>
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				{/if}
			</div>
		</Tooltip.Provider>
	</div>

	<div
		class={cn(
			'min-h-0 overflow-y-auto',
			variant === 'full' ? 'flex-1' : 'max-h-64 rounded-md border'
		)}
	>
		<ContextMenu.Root>
			<ContextMenu.Trigger class="h-full select-text">
				<div
					bind:this={host}
					role="presentation"
					class={variant === 'full' ? 'h-full' : ''}
					oncontextmenu={handleContextMenu}
				></div>
			</ContextMenu.Trigger>
			<ContextMenu.Content class="w-56 text-xs">
				{#if active.table}
					<ContextMenu.Item onSelect={addRowAfter}>
						<span>Ajouter une ligne en dessous</span>
						<ContextMenu.Shortcut>Ctrl+Alt+↓</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={addRowBefore}>
						<span>Ajouter une ligne au-dessus</span>
						<ContextMenu.Shortcut>Ctrl+Alt+↑</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={deleteRow}>
						<span>Supprimer la ligne</span>
					</ContextMenu.Item>
					<ContextMenu.Separator />
					<ContextMenu.Item onSelect={addColAfter}>
						<span>Ajouter une colonne à droite</span>
						<ContextMenu.Shortcut>Ctrl+Alt+→</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={addColBefore}>
						<span>Ajouter une colonne à gauche</span>
						<ContextMenu.Shortcut>Ctrl+Alt+←</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={deleteCol}>
						<span>Supprimer la colonne</span>
					</ContextMenu.Item>
					<ContextMenu.Separator />
					<ContextMenu.Item variant="destructive" onSelect={deleteTable}>
						<span>Supprimer le tableau</span>
					</ContextMenu.Item>
				{:else}
					<ContextMenu.Item onSelect={insertDefaultTable}>
						<span>Insérer un tableau</span>
						<ContextMenu.Shortcut>Ctrl+Alt+T</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Separator />
					<ContextMenu.Item onSelect={() => run((ed) => ed.chain().focus().toggleBold().run())}>
						<span>Gras</span>
						<ContextMenu.Shortcut>Ctrl+B</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={() => run((ed) => ed.chain().focus().toggleItalic().run())}>
						<span>Italique</span>
						<ContextMenu.Shortcut>Ctrl+I</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={() => run((ed) => ed.chain().focus().toggleUnderline().run())}>
						<span>Souligné</span>
						<ContextMenu.Shortcut>Ctrl+U</ContextMenu.Shortcut>
					</ContextMenu.Item>
					<ContextMenu.Item onSelect={() => run((ed) => ed.chain().focus().toggleHeading({ level: 2 }).run())}>
						<span>Titre</span>
						<ContextMenu.Shortcut>Ctrl+Alt+2</ContextMenu.Shortcut>
					</ContextMenu.Item>
				{/if}
			</ContextMenu.Content>
		</ContextMenu.Root>
	</div>
</div>
