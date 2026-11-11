<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { LoaderCircle } from '@lucide/svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
	const app = getAppContext();
	const pathname = $derived(deLocalizeUrl(page.url).pathname);

	$effect(() => {
		if (app.booting) return;
		if (pathname === '/login' && (!app.configured || !app.connected)) {
			void goto('/connection', { replaceState: true });
		} else if (pathname === '/login' && app.currentUser) {
			void goto(`/${app.currentUser.organization_id}`, { replaceState: true });
		}
	});
</script>

{#if app.booting}
	<main class="grid h-dvh place-content-center justify-items-center gap-3 bg-background text-muted-foreground">
		<LoaderCircle size={22} class="animate-spin text-primary" />
		<p class="text-sm">{m.common_loading()}</p>
	</main>
{:else}
	{@render children()}
{/if}
