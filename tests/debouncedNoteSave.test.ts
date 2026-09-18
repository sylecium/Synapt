import { describe, expect, mock, test } from 'bun:test';
import { createDebouncedNoteSave, flushAllDebouncedNotes, registerDebouncedFlush } from '../src/lib/debouncedNoteSave';
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
		saver.destroy();
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
		saver.destroy();
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
		saver.destroy();
	});

	test('flushAllDebouncedNotes persiste les notes actives en attente', async () => {
		const n1 = note('1', 'note 1');
		const n2 = note('2', 'note 2');
		const save1 = mock(async (x: Note) => x);
		const save2 = mock(async (x: Note) => x);
		const saver1 = createDebouncedNoteSave({
			getNote: (id) => (id === n1.id ? n1 : undefined),
			save: save1,
			onSaved: mock(() => {}),
			onError: mock(() => {})
		});
		const saver2 = createDebouncedNoteSave({
			getNote: (id) => (id === n2.id ? n2 : undefined),
			save: save2,
			onSaved: mock(() => {}),
			onError: mock(() => {})
		});
		saver1.schedule(n1);
		saver2.schedule(n2);
		await flushAllDebouncedNotes();
		expect(save1).toHaveBeenCalledTimes(1);
		expect(save2).toHaveBeenCalledTimes(1);

		saver1.destroy();
		saver2.destroy();
	});

	test('destroy retire la note du registre global', async () => {
		const n = note('3', 'note 3');
		const save = mock(async (x: Note) => x);
		const saver = createDebouncedNoteSave({
			getNote: (id) => (id === n.id ? n : undefined),
			save,
			onSaved: mock(() => {}),
			onError: mock(() => {})
		});
		saver.schedule(n);
		saver.destroy();
		await flushAllDebouncedNotes();
		expect(save).not.toHaveBeenCalled();
	});
});

describe('registerDebouncedFlush', () => {
	test('enregistre un flush appelé par flushAllDebouncedNotes', async () => {
		const customFlush = mock(async () => {});
		const unregister = registerDebouncedFlush(customFlush);

		await flushAllDebouncedNotes();
		expect(customFlush).toHaveBeenCalledTimes(1);

		unregister();
	});

	test('désinscrit le flush via la fonction de retour', async () => {
		const customFlush = mock(async () => {});
		const unregister = registerDebouncedFlush(customFlush);

		unregister();
		await flushAllDebouncedNotes();
		expect(customFlush).not.toHaveBeenCalled();
	});
});
