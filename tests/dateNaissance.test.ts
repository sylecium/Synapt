import { describe, expect, test } from 'bun:test';
import {
	daysInMonth,
	monthAllowsDay,
	parseDdMmYyyy,
	yearAllowsDate,
	ymdToDdMmYyyy
} from '../src/lib/dateNaissance';

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

describe('daysInMonth', () => {
	test('septembre a 30 jours', () => {
		expect(daysInMonth(1990, 9)).toBe(30);
	});

	test('février hors bissextile a 28 jours', () => {
		expect(daysInMonth(1990, 2)).toBe(28);
	});

	test('février bissextile a 29 jours', () => {
		expect(daysInMonth(2024, 2)).toBe(29);
	});
});

describe('monthAllowsDay', () => {
	test('le 31 n’est pas en septembre', () => {
		expect(monthAllowsDay(9, 31)).toBe(false);
		expect(monthAllowsDay(1, 31)).toBe(true);
	});

	test('le 29 peut être en février', () => {
		expect(monthAllowsDay(2, 29)).toBe(true);
	});
});

describe('yearAllowsDate', () => {
	test('29 février seulement les années bissextiles', () => {
		expect(yearAllowsDate(2024, 2, 29)).toBe(true);
		expect(yearAllowsDate(2023, 2, 29)).toBe(false);
	});
});
