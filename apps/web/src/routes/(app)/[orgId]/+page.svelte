<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import DashboardGreeting from '#lib/features/dashboard/DashboardGreeting.svelte';
	import DashboardQuickActions from '#lib/features/dashboard/DashboardQuickActions.svelte';
	import DashboardStatus from '#lib/features/dashboard/DashboardStatus.svelte';
	import { getViewPath, type ViewKey } from '#lib/shell/navigation';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);

	function selectView(view: ViewKey) {
		void goto(getViewPath(organizationId, view));
	}

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

{#if app.currentUser}
	<section class="grid gap-6">
		<DashboardGreeting currentUser={app.currentUser} />
		<DashboardQuickActions onSelect={selectView} />
		<DashboardStatus mode={app.mode} onEditConnection={editConnection} />
	</section>
{/if}
