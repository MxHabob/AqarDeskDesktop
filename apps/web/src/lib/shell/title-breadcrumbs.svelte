<script lang="ts">
	import { Building2, ChevronRight } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';

	let { title }: { title: string } = $props();
</script>

<div class="flex min-w-0 items-center gap-2 text-xs" aria-label={m.nav_workspace()}>
	<Building2 size={16} class="shrink-0 text-muted-foreground" />
	<span class="hidden shrink-0 text-muted-foreground sm:inline">{m.app_name()}</span>
	<ChevronRight size={13} class="hidden shrink-0 text-muted-foreground/50 sm:block rtl:rotate-180" />
	<strong class="truncate font-medium text-foreground">{title}</strong>
</div>
