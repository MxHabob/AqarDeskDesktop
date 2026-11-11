import { locales, setLocale } from '#lib/paraglide/runtime';

export type AppLocale = (typeof locales)[number];

export const localeState = $state({
	revision: 0
});

export async function changeAppLocale(locale: AppLocale): Promise<void> {
	await setLocale(locale);
	localeState.revision += 1;
}
