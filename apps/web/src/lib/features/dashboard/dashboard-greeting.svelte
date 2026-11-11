<script lang="ts">
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';

	let { currentUser }: { currentUser: User } = $props();
	const firstName = $derived(currentUser.name.trim().split(/\s+/)[0] || currentUser.name);
</script>

<div class="grid gap-1">
	<h1 class="font-serif text-2xl font-normal tracking-tight text-foreground">
		{m.dashboard_greeting({ name: firstName })}
	</h1>
	<p class="text-xs text-muted-foreground">{m.dashboard_subtitle()}</p>
</div>
