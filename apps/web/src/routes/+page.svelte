<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';
	import { Building2, LoaderCircle } from '@lucide/svelte';

	const app = getAppContext();

	onMount(() => {
		void app.initialize().then(() =>
			goto(
				app.currentUser
					? `/${app.currentUser.organization_id}`
					: app.configured
						? '/login'
						: '/connection',
				{
				replaceState: true
				}
			)
		);
	});
</script>

<main class="grid h-dvh place-content-center justify-items-center gap-3 bg-slate-50 font-sans text-slate-700">
	<Building2 size={26} class="text-emerald-800" />
	<LoaderCircle size={22} class="animate-spin text-emerald-800" />
	<p class="text-sm">{m.common_loading()}</p>
</main>
