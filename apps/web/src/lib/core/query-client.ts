import { QueryClient } from '@tanstack/svelte-query';

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 60_000,
			gcTime: 5 * 60_000,
			notifyOnChangeProps: 'all',
			refetchOnWindowFocus: false,
			refetchOnReconnect: false,
			retry: 1
		}
	}
});
