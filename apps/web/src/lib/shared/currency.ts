const zeroDecimalCurrencies = new Set([
	'BIF', 'CLP', 'DJF', 'GNF', 'ISK', 'JPY', 'KMF', 'KRW', 'PYG', 'RWF', 'UGX', 'VND', 'VUV', 'XAF',
	'XOF', 'XPF'
]);
const threeDecimalCurrencies = new Set(['BHD', 'IQD', 'JOD', 'KWD', 'LYD', 'OMR', 'TND']);

export function currencyDecimalDigits(currencyCode: string): number {
	const code = currencyCode.toUpperCase();
	if (zeroDecimalCurrencies.has(code)) return 0;
	if (threeDecimalCurrencies.has(code)) return 3;
	return 2;
}

export function formatMinorAmount(amountMinor: number | null, currencyCode: string): string {
	if (amountMinor === null) return '';
	const digits = currencyDecimalDigits(currencyCode);
	return (amountMinor / 10 ** digits).toFixed(digits);
}

export function parseMajorAmount(amount: string, currencyCode: string): number | null {
	if (!amount.trim()) return null;
	const amountMinor = Math.round(Number(amount) * 10 ** currencyDecimalDigits(currencyCode));
	return Number.isSafeInteger(amountMinor) && amountMinor >= 0 ? amountMinor : null;
}
