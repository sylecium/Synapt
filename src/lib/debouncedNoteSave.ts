import type { Note } from './types';

const DELAY_MS = 400;

const activeFlushes = new Set<() => Promise<void>>();

export async function flushAllDebouncedNotes(): Promise<void> {
	await Promise.all(Array.from(activeFlushes).map((flush) => flush().catch(() => {})));
}

export function createDebouncedNoteSave(opts: {
	getNote: (id: string) => Note | undefined;
	save: (note: Note) => Promise<Note>;
	onSaved: (note: Note) => void;
	onError: (e: unknown) => void;
}) {
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let pendingId: string | null = null;

	async function flush() {
		if (saveTimer) {
			clearTimeout(saveTimer);
			saveTimer = null;
		}
		const id = pendingId;
		if (!id) return;
		const note = opts.getNote(id);
		pendingId = null;
		if (!note) return;
		try {
			const saved = await opts.save(note);
			note.updated_at = saved.updated_at;
			opts.onSaved(note);
		} catch (e) {
			opts.onError(e);
		}
	}

	activeFlushes.add(flush);

	function schedule(note: Note) {
		pendingId = note.id;
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			void flush();
		}, DELAY_MS);
	}

	function cancel(id?: string) {
		if (id !== undefined && pendingId !== id) return;
		if (saveTimer) {
			clearTimeout(saveTimer);
			saveTimer = null;
		}
		pendingId = null;
	}

	function destroy() {
		activeFlushes.delete(flush);
	}

	return { flush, schedule, cancel, destroy };
}
