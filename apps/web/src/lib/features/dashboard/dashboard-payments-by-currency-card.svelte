<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { Banknote } from '@lucide/svelte';
	import { formatCurrency, formatNumber } from '#lib/core/i18n';
	import type { DashboardSummary } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();

</script>

<Card class="flex h-[182px] min-w-0 flex-col p-3">
	<div class="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
		<Banknote size={14} />
		<h3>{m.dashboard_payments_by_currency()}</h3>
		<strong class="ms-auto font-medium text-foreground">
			{formatNumber(
				summary.payments_by_currency.reduce((total, item) => total + item.payment_count, 0),
				{ maximumFractionDigits: 0 }
			)}
		</strong>
	</div>
	{#if summary.payments_by_currency.length === 0}
		<p class="grid min-h-0 flex-1 place-items-center text-xs text-muted-foreground">
			{m.dashboard_no_payments()}
		</p>
	{:else}
		<div class="grid min-h-0 flex-1 content-start gap-2 overflow-y-auto scrollbar-none">
			{#each summary.payments_by_currency as total (total.currency_code)}
				<div class="flex items-center justify-between gap-3 text-xs">
					<span class="text-muted-foreground">{total.currency_code}</span>
					<strong class="font-medium text-foreground">
						{formatCurrency(
							total.amount_minor,
							total.currency_code,
							total.currency_decimal_digits
						)}
					</strong>
				</div>
			{/each}
		</div>
	{/if}
</Card>
