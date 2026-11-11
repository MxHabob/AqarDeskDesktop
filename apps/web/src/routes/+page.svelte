<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';
	import { LoaderCircle } from '@lucide/svelte';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';

	const app = getAppContext();

	$effect(() => {
		if (app.booting) return;
		void goto(
			app.currentUser
				? `/${app.currentUser.organization_id}`
				: app.configured && app.connected
					? '/login'
					: '/connection',
			{ replaceState: true }
		);
	});
</script>

<main class="grid h-dvh place-content-center justify-items-center gap-3 bg-background font-sans text-foreground">
	<AqarDeskLogo size={26} class="text-primary" />
	<LoaderCircle size={22} class="animate-spin text-emerald-800" />
	<p class="text-sm">{m.common_loading()}</p>
</main>
