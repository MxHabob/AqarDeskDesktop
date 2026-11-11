<script lang="ts">
	import { Building2 } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';

	let { currentUser }: { currentUser: User } = $props();
</script>

<header class="relative flex items-center justify-between gap-5 overflow-hidden rounded-2xl border border-border/70 bg-card p-5 shadow-card sm:p-8">
	<div class="relative z-10 grid gap-2">
		<p class="text-[11px] font-semibold uppercase tracking-widest text-primary">{m.nav_overview()}</p>
		<h1 class="text-2xl font-semibold tracking-tight text-foreground sm:text-3xl">
			{m.dashboard_greeting({ name: currentUser.name })}
		</h1>
		<p class="max-w-2xl text-sm leading-relaxed text-muted-foreground">{m.dashboard_subtitle()}</p>
	</div>
	<span class="relative hidden size-20 shrink-0 place-items-center rounded-2xl bg-accent text-accent-foreground sm:grid">
		<span class="absolute -end-3 -top-3 size-14 rounded-full bg-primary/10"></span>
		<Building2 size={32} class="relative" />
	</span>
</header>
