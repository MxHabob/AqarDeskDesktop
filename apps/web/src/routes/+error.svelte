<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { Building2 } from '@lucide/svelte';

	const retry = () => window.location.reload();
</script>

<main class="grid h-dvh place-content-center justify-items-center gap-5 bg-background px-6 text-center text-foreground">
	<Building2 size={28} class="text-primary" />
	<div class="grid max-w-md gap-2">
		<h1 class="text-xl font-semibold">{m.error_unexpected_title()}</h1>
		<p class="text-sm leading-relaxed text-muted-foreground">{m.error_unexpected_description()}</p>
	</div>
	<div class="flex flex-wrap justify-center gap-2">
		<Button onclick={retry}>{m.common_retry()}</Button>
		<Button variant="outline" onclick={() => void goto('/', { replaceState: true })}>
			{m.app_name()}
		</Button>
	</div>
</main>
