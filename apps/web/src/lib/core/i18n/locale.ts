import { getLocale } from '#lib/paraglide/runtime';

function isPresent<T>(value: T): value is Exclude<T, null | undefined> {
	return value !== null && value !== undefined && value !== '';
}

export function l<T>(arValue: T, enValue: T): T {
	return getLocale() === 'ar'
		? isPresent(arValue)
			? arValue
			: enValue
		: isPresent(enValue)
			? enValue
			: arValue;
}

export function isAr(): boolean {
	return getLocale() === 'ar';
}
