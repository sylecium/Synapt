import { describe, expect, test } from 'bun:test';
import {
	noteIsEmpty,
	notePlainText,
	noteTitle,
	noteToEditorContent
} from '../src/lib/notesHtml';

describe('noteToEditorContent', () => {
	test('enveloppe le texte brut en paragraphes', () => {
		expect(noteToEditorContent('ligne 1\nligne 2')).toBe('<p>ligne 1</p><p>ligne 2</p>');
	});

	test('laisse le HTML sûr inchangé', () => {
		const html = '<p>déjà <strong>formaté</strong></p>';
		expect(noteToEditorContent(html)).toBe(html);
	});

	test('strippe le HTML dangereux', () => {
		expect(noteToEditorContent('<p>ok</p><script>alert(1)</script>')).toBe('<p>ok</p>');
		expect(noteToEditorContent('<p onclick="alert(1)">x</p>')).toBe('<p>x</p>');
		expect(noteToEditorContent('<a href="javascript:alert(1)">x</a>')).toBe('x');
	});

	test('vide devient un paragraphe vide', () => {
		expect(noteToEditorContent('')).toBe('<p></p>');
	});
});

describe('notePlainText / noteTitle / noteIsEmpty', () => {
	test('extrait le texte d’un HTML', () => {
		expect(notePlainText('<p>Bonjour</p><p>suite</p>')).toBe('Bonjour\nsuite');
	});

	test('titre = première ligne', () => {
		expect(noteTitle('<p>Titre</p><p>corps</p>')).toBe('Titre');
	});

	test('note HTML vide est vide', () => {
		expect(noteIsEmpty('<p></p>')).toBe(true);
		expect(noteIsEmpty(null)).toBe(true);
		expect(noteIsEmpty('<p>ok</p>')).toBe(false);
	});
});
