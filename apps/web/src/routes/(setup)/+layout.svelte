<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { onMount } from 'svelte';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
	const app = getAppContext();

	onMount(() => {
		void app.initialize();
	});

	$effect(() => {
		if (app.booting) return;
		if (page.url.pathname === '/login' && !app.configured) {
			void goto('/connection', { replaceState: true });
		} else if (page.url.pathname === '/login' && app.currentUser) {
			void goto(`/${app.currentUser.organization_id}`, { replaceState: true });
		}
	});
</script>

{@render children()}
