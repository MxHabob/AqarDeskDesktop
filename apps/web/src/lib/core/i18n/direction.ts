import { getLocale, getTextDirection } from '#lib/paraglide/runtime';

export type HtmlDirection = 'ltr' | 'rtl';
export type PhysicalSide = 'left' | 'right';

export function getHtmlDirection(): HtmlDirection {
	if (typeof document !== 'undefined') {
		const direction = document.documentElement.dir;
		if (direction === 'ltr' || direction === 'rtl') return direction;
	}

	return getTextDirection(getLocale());
}

export function getInlineEndSide(direction = getHtmlDirection()): PhysicalSide {
	return direction === 'rtl' ? 'left' : 'right';
}
