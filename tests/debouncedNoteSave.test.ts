import { describe, expect, mock, test } from 'bun:test';
import { createDebouncedNoteSave } from '../src/lib/debouncedNoteSave';
import type { Note } from '../src/lib/types';

function note(id: string, corps = 'x'): Note {
	return { id, client_id: null, corps, created_at: 't0', updated_at: 't0' };
}

describe('createDebouncedNoteSave', () => {
	test('flush sans pending ne sauvegarde pas', async () => {
		const save = mock(async (n: Note) => n);
		const saver = createDebouncedNoteSave({
			getNote: () => undefined,
			save,
			onSaved: mock(() => {}),
			onError: mock(() => {})
		});
		await saver.flush();
		expect(save).not.toHaveBeenCalled();
	});

	test('flush immédiat persiste et met à jour updated_at', async () => {
		const n = note('1', 'hello');
		const save = mock(async (x: Note) => ({ ...x, updated_at: 't2' }));
		const onSaved = mock(() => {});
		const saver = createDebouncedNoteSave({
			getNote: (id) => (id === n.id ? n : undefined),
			save,
			onSaved,
			onError: mock(() => {})
		});
		saver.schedule(n);
		await saver.flush();
		expect(save).toHaveBeenCalledTimes(1);
		expect(n.updated_at).toBe('t2');
		expect(onSaved).toHaveBeenCalledTimes(1);
	});

	test('cancel abandonne le pending', async () => {
		const n = note('1');
		const save = mock(async (x: Note) => x);
		const saver = createDebouncedNoteSave({
			getNote: (id) => (id === n.id ? n : undefined),
			save,
			onSaved: mock(() => {}),
			onError: mock(() => {})
		});
		saver.schedule(n);
		saver.cancel(n.id);
		await saver.flush();
		expect(save).not.toHaveBeenCalled();
	});
});
