<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import { queryClient } from '#lib/core/query-client';
	import ConfirmDialog from '#lib/shared/components/ConfirmDialog.svelte';
	import { createAppContext, setAppContext } from '#lib/core/app-context.svelte';
	import type { UpdateProgress } from '#lib/api/desktop';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
	const app = createAppContext();
	setAppContext(app);

	onMount(() => {
		void app.initialize();
		let disposed = false;
		let unlisten: UnlistenFn | undefined;
		void listen<UpdateProgress>('desktop-update-progress', (event) => {
			app.updateProgress = event.payload;
		})
			.then((stopListening) => {
				if (disposed) stopListening();
				else unlisten = stopListening;
			})
			.catch(app.setError);

		return () => {
			disposed = true;
			unlisten?.();
		};
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<QueryClientProvider client={queryClient}>
	{@render children()}
</QueryClientProvider>
<ConfirmDialog />
