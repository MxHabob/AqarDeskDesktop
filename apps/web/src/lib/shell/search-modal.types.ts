import type { ViewKey } from './navigation';

export type SearchItem = {
	key: ViewKey;
	label: string;
	keywords: string[];
};
