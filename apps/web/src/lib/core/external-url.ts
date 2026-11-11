import { openUrl } from '@tauri-apps/plugin-opener';

export async function openExternalUrl(value: string): Promise<void> {
	const url = new URL(value);
	const isWebUrl = url.protocol === 'https:' || url.protocol === 'http:';
	const isMailtoUrl =
		url.protocol === 'mailto:' &&
		/^[\w.+-]+@[\w.-]+\.[A-Za-z]{2,}$/.test(decodeURIComponent(url.pathname));
	if (!isWebUrl && !isMailtoUrl) {
		throw new Error('Only HTTP, HTTPS, and valid mailto links can be opened externally.');
	}

	if ('__TAURI_INTERNALS__' in window) {
		await openUrl(url.href);
		return;
	}

	window.open(url.href, '_blank', 'noopener,noreferrer');
}
