<script lang="ts">
	import { onMount } from 'svelte';
	import type { User } from '#lib/features/auth/types';
	import * as m from '#lib/paraglide/messages';

	let { currentUser, timeZone }: { currentUser: User; timeZone: string } = $props();
	let now = $state(new Date());
	const firstName = $derived(currentUser.name.trim().split(/\s+/)[0] || currentUser.name);
	const hour = $derived.by(() => {
		try {
			return Number(
				new Intl.DateTimeFormat('en-US', {
					timeZone,
					hour: '2-digit',
					hourCycle: 'h23'
				})
					.formatToParts(now)
					.find((part) => part.type === 'hour')?.value ?? '0'
			);
		} catch {
			return now.getHours();
		}
	});
	const greeting = $derived(
		hour >= 5 && hour < 12
			? m.dashboard_greeting_morning()
			: hour >= 12 && hour < 17
				? m.dashboard_greeting_afternoon()
				: hour >= 17 && hour < 22
					? m.dashboard_greeting_evening()
					: m.dashboard_greeting_night()
	);

	onMount(() => {
		const interval = window.setInterval(() => {
			now = new Date();
		}, 60_000);
		return () => window.clearInterval(interval);
	});
</script>

<div class="grid gap-1">
	<h1 class="font-serif text-3xl font-normal tracking-tight text-foreground">
		{greeting}
		<span class="text-muted-foreground">{firstName}</span>
	</h1>
	<p class="text-xs text-muted-foreground">{m.dashboard_subtitle()}</p>
</div>
