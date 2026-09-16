import type { Note } from './types';

const DELAY_MS = 400;

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

	return { flush, schedule, cancel };
}
