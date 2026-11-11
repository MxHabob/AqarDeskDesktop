<script lang="ts">
	import { Building2 } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';

	let { currentUser }: { currentUser: User } = $props();
</script>

<header class="flex items-center justify-between gap-5 rounded-xl border border-border bg-card p-5 shadow-card sm:p-8">
	<div class="grid gap-2">
		<p class="text-xs font-semibold tracking-wide text-muted-foreground">{m.nav_overview()}</p>
		<h1 class="text-2xl font-semibold tracking-tight text-foreground sm:text-3xl">
			{m.dashboard_greeting({ name: currentUser.name })}
		</h1>
		<p class="max-w-2xl text-sm leading-relaxed text-muted-foreground">{m.dashboard_subtitle()}</p>
	</div>
	<span class="hidden size-16 shrink-0 place-items-center rounded-xl bg-accent text-accent-foreground shadow-sm sm:grid">
		<Building2 size={30} />
	</span>
</header>
