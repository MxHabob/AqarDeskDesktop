import { openUrl } from '@tauri-apps/plugin-opener';

export async function openExternalUrl(value: string): Promise<void> {
	const url = new URL(value);
	if (url.protocol !== 'https:' && url.protocol !== 'http:') {
		throw new Error('Only HTTP and HTTPS links can be opened externally.');
	}

	if ('__TAURI_INTERNALS__' in window) {
		await openUrl(url.href);
		return;
	}

	window.open(url.href, '_blank', 'noopener,noreferrer');
}
