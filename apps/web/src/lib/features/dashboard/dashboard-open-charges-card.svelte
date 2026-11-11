<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { FileClock } from '@lucide/svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import type { DashboardSummary } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();

	function formatAmount(amountMinor: number, currency: string, digits: number) {
		return new Intl.NumberFormat(getLocale(), {
			style: 'currency',
			currency,
			maximumFractionDigits: digits
		}).format(amountMinor / 10 ** digits);
	}
</script>

<Card class="flex h-[182px] min-w-0 flex-col p-3">
	<div class="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
		<FileClock size={14} />
		<h3>{m.dashboard_open_charge_balances()}</h3>
		<strong class="ms-auto font-medium text-foreground">{summary.open_charges_count}</strong>
	</div>
	{#if summary.open_charges_by_currency.length === 0}
		<p class="grid min-h-0 flex-1 place-items-center text-xs text-muted-foreground">
			{m.dashboard_no_open_charges()}
		</p>
	{:else}
		<div class="grid min-h-0 flex-1 content-start gap-2 overflow-y-auto scrollbar-none">
			{#each summary.open_charges_by_currency as balance (balance.currency_code)}
				<div class="flex items-center justify-between gap-3 text-xs">
					<span class="text-muted-foreground">{balance.currency_code}</span>
					<strong class="font-medium text-foreground">
						{formatAmount(balance.amount_minor, balance.currency_code, balance.currency_decimal_digits)}
					</strong>
				</div>
			{/each}
		</div>
	{/if}
</Card>
