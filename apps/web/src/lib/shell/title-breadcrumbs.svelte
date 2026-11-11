<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';
	import * as m from '#lib/paraglide/messages';
	import { getViewPath } from '#lib/shell/navigation';

	let {
		title,
		organizationName,
		organizationId
	}: {
		title: string;
		organizationName: string;
		organizationId: string | null;
	} = $props();
	const organizationLabel = $derived(organizationName.trim() || m.app_name());
</script>

<div class="flex min-w-0 items-center gap-2 text-xs" aria-label={m.nav_workspace()}>
	<a
		href={organizationId ? getViewPath(organizationId, 'overview') : '/'}
		class="flex min-w-0 items-center gap-2 rounded-sm outline-none hover:text-primary focus-visible:ring-2 focus-visible:ring-ring"
		data-tauri-drag-region="false"
		aria-label={m.nav_overview()}
		title={m.nav_overview()}
	>
		<AqarDeskLogo size={25} class="text-foreground" />
		<span class="hidden max-w-48 truncate text-muted-foreground sm:inline" title={organizationLabel}>
			{organizationLabel}
		</span>
	</a>
	<ChevronRight size={13} class="hidden shrink-0 text-muted-foreground/50 sm:block rtl:rotate-180" />
	<strong class="truncate font-medium text-foreground">{title}</strong>
</div>
