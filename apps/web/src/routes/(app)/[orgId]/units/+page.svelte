<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ModulePlaceholder from '#lib/features/dashboard/ModulePlaceholder.svelte';
	import { getViewPath } from '#lib/shell/navigation';
	import * as m from '#lib/paraglide/messages';
	import PageHeader from '#lib/shared/components/PageHeader.svelte';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
</script>

{#if app.currentUser}
	<PageHeader title={m.nav_units()} description={m.dashboard_properties_description()} eyebrow={m.nav_workspace()} />
	<ModulePlaceholder title={m.nav_units()} description={m.dashboard_properties_description()} currentUser={app.currentUser} onSelect={(view) => void goto(getViewPath(organizationId, view))} />
{/if}
