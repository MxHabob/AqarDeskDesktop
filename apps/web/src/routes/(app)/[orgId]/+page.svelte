<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import DashboardGreeting from '#lib/features/dashboard/dashboard-greeting.svelte';
	import DashboardQuickActions from '#lib/features/dashboard/dashboard-quick-actions.svelte';
	import DashboardStatistics from '#lib/features/dashboard/dashboard-statistics.svelte';
	import DashboardToolbar from '#lib/features/dashboard/dashboard-toolbar.svelte';
	import { getViewPath, type ViewKey } from '#lib/shell/navigation';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	let selectedMonths = $state(12);
	let dashboardView = $state<'overview' | 'metrics'>('overview');

	function selectView(view: ViewKey) {
		void goto(getViewPath(organizationId, view));
	}

</script>

{#if app.currentUser}
	<section class="grid w-full gap-4">
		<header class="flex min-h-10 flex-wrap items-start justify-between gap-3 sm:flex-nowrap sm:items-center">
			<DashboardGreeting currentUser={app.currentUser} timeZone={app.organizationTimezone} />
			<DashboardToolbar
				months={selectedMonths}
				view={dashboardView}
				onMonthsChange={(months) => (selectedMonths = months)}
				onViewChange={(view) => (dashboardView = view)}
				onSettings={() => selectView('settings')}
			/>
		</header>
		<DashboardStatistics currentUser={app.currentUser} months={selectedMonths} view={dashboardView} />
		<DashboardQuickActions onSelect={selectView} />
	</section>
{/if}
