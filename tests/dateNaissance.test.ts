import { describe, expect, test } from 'bun:test';
import { parseDdMmYyyy, ymdToDdMmYyyy } from '../src/lib/dateNaissance';

describe('parseDdMmYyyy', () => {
	test('accepte jj/mm/aaaa', () => {
		expect(parseDdMmYyyy('15/09/1990')).toBe('1990-09-15');
	});

	test('accepte un chiffre sans zéro', () => {
		expect(parseDdMmYyyy('1/2/1990')).toBe('1990-02-01');
	});

	test('rejette une date impossible', () => {
		expect(parseDdMmYyyy('31/02/1990')).toBeNull();
	});

	test('rejette un champ vide', () => {
		expect(parseDdMmYyyy('')).toBeNull();
	});
});

describe('ymdToDdMmYyyy', () => {
	test('formate ISO vers jj/mm/aaaa', () => {
		expect(ymdToDdMmYyyy('1990-09-15')).toBe('15/09/1990');
	});
});
