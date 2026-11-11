import { getLocale } from '#lib/paraglide/runtime';
import {
	formatCell as formatGridCellFallback,
	getCellValue,
	type GridRow,
	type OrbisColumnDef
} from '@aqardesk/svelte-grid';

type DateFormat = 'date' | 'datetime' | 'time';

const numberFormatters = new Map<string, Intl.NumberFormat>();
const dateFormatters = new Map<string, Intl.DateTimeFormat>();

function numberFormatter(options: Intl.NumberFormatOptions): Intl.NumberFormat {
	const locale = getLocale() === 'ar' ? 'ar-SA' : 'en-US';
	const key = `${locale}:${JSON.stringify(options)}`;
	let formatter = numberFormatters.get(key);
	if (!formatter) {
		formatter = new Intl.NumberFormat(locale, options);
		numberFormatters.set(key, formatter);
	}
	return formatter;
}

function dateFormatter(
	options: Intl.DateTimeFormatOptions,
	timeZone?: string
): Intl.DateTimeFormat {
	const locale = getLocale() === 'ar' ? 'ar-SA' : 'en-US';
	const resolvedTimeZone = timeZone ?? Intl.DateTimeFormat().resolvedOptions().timeZone;
	const normalizedOptions = { ...options, timeZone: resolvedTimeZone };
	const key = `${locale}:${JSON.stringify(normalizedOptions)}`;
	let formatter = dateFormatters.get(key);
	if (!formatter) {
		formatter = new Intl.DateTimeFormat(locale, normalizedOptions);
		dateFormatters.set(key, formatter);
	}
	return formatter;
}

export function formatNumber(
	value: number | null | undefined,
	options: Intl.NumberFormatOptions = { maximumFractionDigits: 2 }
): string {
	if (value === null || value === undefined) return '';
	return numberFormatter(options).format(value);
}

function formatCurrencyValue(value: number, currency: string, decimalDigits: number): string {
	const digits = Math.max(0, Math.min(4, decimalDigits));
	return numberFormatter({
		style: 'currency',
		currency: currency.trim().toUpperCase(),
		minimumFractionDigits: digits,
		maximumFractionDigits: digits
	}).format(value);
}

export function formatCurrency(
	amountMinor: number | null | undefined,
	currency: string,
	decimalDigits: number
): string {
	if (amountMinor === null || amountMinor === undefined) return '';
	const digits = Math.max(0, Math.min(4, decimalDigits));
	return formatCurrencyValue(amountMinor / 10 ** digits, currency, digits);
}

export function formatDate(
	value: string | number | Date | null | undefined,
	type: DateFormat = 'date',
	options?: Intl.DateTimeFormatOptions,
	timeZone?: string
): string {
	if (value === null || value === undefined || value === '') return '';
	const dateOnly = typeof value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(value);
	const date = value instanceof Date ? value : new Date(value);
	if (Number.isNaN(date.getTime())) return '';

	const defaultOptions: Intl.DateTimeFormatOptions =
		type === 'datetime'
			? { dateStyle: 'medium', timeStyle: 'short' }
			: type === 'time'
				? { hour: 'numeric', minute: '2-digit' }
				: { dateStyle: 'medium' };
	return dateFormatter(
		{ ...defaultOptions, ...options },
		timeZone ?? options?.timeZone ?? (dateOnly ? 'UTC' : undefined)
	).format(date);
}

export function formatMonthLabel(month: string): string {
	const match = /^(\d{4})-(\d{2})$/.exec(month);
	if (!match) return month;
	const year = Number(match[1]);
	const monthIndex = Number(match[2]) - 1;
	const date = new Date(Date.UTC(year, monthIndex, 1));
	if (date.getUTCMonth() !== monthIndex || date.getUTCFullYear() !== year) return month;
	return dateFormatter({ month: 'short' }, 'UTC').format(date);
}

export function formatGridCell(
	row: GridRow,
	column: OrbisColumnDef,
	timeZone?: string
): string {
	const value = getCellValue(row, column);
	if (value === null || value === undefined || value === '') return '';

	if (typeof value === 'number') {
		switch (column.type) {
			case 'currency':
				return formatCurrencyValue(value, column.currency ?? 'USD', 2);
			case 'percentage':
				return `${formatNumber(value, { maximumFractionDigits: 1 })}%`;
			case 'integer':
				return formatNumber(value, { maximumFractionDigits: 0 });
			case 'number':
			case 'decimal':
				return formatNumber(value);
		}
	}

	if (
		(column.type === 'date' || column.type === 'datetime') &&
		(typeof value === 'string' || value instanceof Date)
	) {
		return (
			formatDate(value, column.type, undefined, timeZone) ||
			(value instanceof Date ? value.toISOString() : value)
		);
	}

	return formatGridCellFallback(row, column, getLocale() === 'ar' ? 'ar' : 'en');
}
