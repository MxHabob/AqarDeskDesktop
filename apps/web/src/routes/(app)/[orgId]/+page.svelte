<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import DashboardGreeting from '#lib/features/dashboard/dashboard-greeting.svelte';
	import DashboardQuickActions from '#lib/features/dashboard/dashboard-quick-actions.svelte';
	import DashboardStatus from '#lib/features/dashboard/dashboard-status.svelte';
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
	<section class="mx-auto grid w-full max-w-5xl gap-6">
		<DashboardGreeting currentUser={app.currentUser} />
		<DashboardStatus mode={app.mode} onEditConnection={editConnection} />
		<DashboardQuickActions onSelect={selectView} />
	</section>
{/if}
