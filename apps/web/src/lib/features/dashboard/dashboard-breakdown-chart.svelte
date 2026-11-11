<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import type { DashboardSummary } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();
	const statuses = $derived(summary.unit_status_counts);
	const maxCount = $derived(Math.max(0, ...statuses.map((status) => status.count)));

	function statusLabel(status: string) {
		switch (status) {
			case 'available': return m.dashboard_status_available();
			case 'maintenance': return m.dashboard_status_maintenance();
			case 'inactive': return m.dashboard_status_inactive();
			case 'draft': return m.dashboard_status_draft();
			case 'pending': return m.dashboard_status_pending();
			case 'active': return m.dashboard_status_active();
			case 'expired': return m.dashboard_status_expired();
			case 'terminated': return m.dashboard_status_terminated();
			case 'void': return m.dashboard_status_void();
			default: return status;
		}
	}
</script>

<Card class="flex h-[182px] min-w-0 flex-col p-3">
	<div class="mb-2 flex items-center justify-between gap-3">
		<h3 class="text-xs font-medium text-muted-foreground">{m.dashboard_status_breakdown()}</h3>
		<span class="text-[11px] text-muted-foreground">{m.dashboard_unit_statuses()}</span>
	</div>

	<div class="grid min-h-0 flex-1 content-center gap-2 overflow-y-auto scrollbar-none">
		{#if statuses.length === 0}
			<p class="text-center text-xs text-muted-foreground">{m.dashboard_no_status_records()}</p>
		{:else}
			{#each statuses as item (item.status)}
				<div class="grid grid-cols-[minmax(4rem,auto)_1fr_auto] items-center gap-3 text-xs">
					<span class="truncate text-muted-foreground">{statusLabel(item.status)}</span>
					<div class="h-1.5 bg-muted">
						<div
							class="h-full bg-foreground/80"
							style={`width: ${maxCount === 0 ? 0 : (item.count / maxCount) * 100}%`}
						></div>
					</div>
					<strong class="min-w-5 text-end font-medium text-foreground">{item.count}</strong>
				</div>
			{/each}
		{/if}
	</div>
</Card>
