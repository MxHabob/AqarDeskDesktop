<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { Building2, Files, House, UsersRound, FileText } from '@lucide/svelte';
	import { Card } from '@aqardesk/ui';
	import type { User } from '#lib/features/auth/types';
	import { dashboardQueries, dashboardQueryKeys } from '#lib/features/dashboard/dashboard-queries';
	import DashboardBreakdownChart from '#lib/features/dashboard/dashboard-breakdown-chart.svelte';
	import DashboardMonthlyPaymentsChart from '#lib/features/dashboard/dashboard-monthly-payments-chart.svelte';
	import DashboardOpenChargesCard from '#lib/features/dashboard/dashboard-open-charges-card.svelte';
	import DashboardStatCard from '#lib/features/dashboard/dashboard-stat-card.svelte';
	import DashboardStackedSummary from '#lib/features/dashboard/dashboard-stacked-summary.svelte';
	import * as m from '#lib/paraglide/messages';
	import { toast } from '@aqardesk/ui';

	let {
		currentUser,
		months,
		view
	}: {
		currentUser: User;
		months: number;
		view: 'overview' | 'metrics';
	} = $props();

	const summaryQuery = createQuery(() => ({
		queryKey: dashboardQueryKeys.summary(currentUser.organization_id, months),
		queryFn: () => dashboardQueries.summary(months)
	}));

	let lastNotifiedError = '';
	$effect(() => {
		const error = summaryQuery.error ? String(summaryQuery.error) : '';
		if (error && error !== lastNotifiedError) toast.error(error);
		lastNotifiedError = error;
	});

	const cards = $derived(
		summaryQuery.data
			? {
					primary: [
						{
							label: m.dashboard_properties_count(),
							value: summaryQuery.data.active_properties_count,
							icon: Building2
						},
						{
							label: m.dashboard_units_count(),
							value: summaryQuery.data.units_count,
							icon: House,
							description: m.dashboard_available_units_hint({
								count: summaryQuery.data.available_units_count
							})
						},
						{
							label: m.dashboard_tenants_count(),
							value: summaryQuery.data.tenants_count,
							icon: UsersRound
						},
						{
							label: m.dashboard_active_leases_count(),
							value: summaryQuery.data.active_leases_count,
							icon: FileText
						}
					],
					documents: {
						label: m.dashboard_contract_documents_count(),
						value: summaryQuery.data.contract_documents_count,
						icon: Files,
						description: m.dashboard_expiring_leases_hint({
							count: summaryQuery.data.expiring_leases_count
						})
					}
				}
			: null
	);
</script>

<section class="grid gap-3" aria-labelledby="dashboard-statistics-title">
	<h2 id="dashboard-statistics-title" class="sr-only">{m.dashboard_statistics_title()}</h2>

	{#if summaryQuery.isLoading && !summaryQuery.data}
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4" aria-live="polite">
			{#each [0, 1, 2, 3] as index (index)}
				<Card class="h-[166px] animate-pulse p-3">
					<div class="h-3 w-24 rounded bg-muted"></div>
					<div class="mt-8 h-7 w-16 rounded bg-muted"></div>
				</Card>
			{/each}
		</div>
	{:else if summaryQuery.data && cards}
		<DashboardStackedSummary summary={summaryQuery.data} />
		{#if view === 'overview'}
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
				{#each cards.primary as card (card.label)}
					<DashboardStatCard {...card} />
				{/each}
				<DashboardMonthlyPaymentsChart summary={summaryQuery.data} />
				<DashboardBreakdownChart summary={summaryQuery.data} />
				<DashboardOpenChargesCard summary={summaryQuery.data} />
				<DashboardStatCard {...cards.documents} />
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
				<div class="lg:col-span-2">
					<DashboardMonthlyPaymentsChart summary={summaryQuery.data} />
				</div>
				<div class="lg:col-span-2">
					<DashboardBreakdownChart summary={summaryQuery.data} />
				</div>
				{#each cards.primary as card (card.label)}
					<DashboardStatCard {...card} />
				{/each}
				<div class="lg:col-span-2">
					<DashboardOpenChargesCard summary={summaryQuery.data} />
				</div>
				<div class="lg:col-span-2">
					<DashboardStatCard {...cards.documents} />
				</div>
			</div>
		{/if}
	{/if}
</section>
