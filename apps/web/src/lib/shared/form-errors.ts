import * as m from '#lib/paraglide/messages';

export function getFieldError(errors: unknown[]): string | undefined {
	const first = errors[0];
	if (!first) return undefined;
	if (typeof first === 'string') return first;
	if (first instanceof Error) return first.message;
	if (typeof first === 'object' && 'message' in first && typeof first.message === 'string') {
		return first.message;
	}
	return m.error_unexpected_description();
}
