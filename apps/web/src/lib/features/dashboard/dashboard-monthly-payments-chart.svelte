<script lang="ts">
	import {
		Card,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue
	} from '@aqardesk/ui';
	import { formatCurrency, formatMonthLabel, formatNumber } from '#lib/core/i18n';
	import type { DashboardSummary } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();
	let selectedCurrency = $state('');

	const currencies = $derived(
		[...new Set(summary.monthly_payments.map((payment) => payment.currency_code))].sort()
	);
	const activeCurrency = $derived(currencies.includes(selectedCurrency) ? selectedCurrency : (currencies[0] ?? ''));
	const currencyDigits = $derived(
		summary.monthly_payments.find((payment) => payment.currency_code === activeCurrency)
			?.currency_decimal_digits ?? 2
	);
	const values = $derived(
		summary.payment_months.map((month) => ({
			month,
			amount:
				summary.monthly_payments.find(
					(payment) => payment.month === month && payment.currency_code === activeCurrency
				)?.amount_minor ?? 0
		}))
	);
	const maxAmount = $derived(Math.max(0, ...values.map((entry) => entry.amount)));

</script>

<Card class="flex h-[182px] min-w-0 flex-col p-3">
	<div class="mb-2 flex items-center justify-between gap-3">
		<h3 class="text-xs font-medium text-muted-foreground">{m.dashboard_payment_trend()}</h3>
		{#if currencies.length > 0}
			<Select
				type="single"
				value={activeCurrency}
				onValueChange={(value) => (selectedCurrency = value)}
			>
				<SelectTrigger class="h-7 w-24 rounded-sm border-border/70 px-2">
					<SelectValue aria-label={m.dashboard_payment_trend()} />
				</SelectTrigger>
				<SelectContent>
					{#each currencies as currency (currency)}
						<SelectItem value={currency}>{currency}</SelectItem>
					{/each}
				</SelectContent>
			</Select>
		{/if}
	</div>

	{#if !activeCurrency}
		<p class="grid min-h-24 flex-1 place-items-center text-xs text-muted-foreground">{m.dashboard_no_payments()}</p>
	{:else}
		<div
			class="flex h-20 items-end justify-between gap-2 border-b border-border/60 px-1"
			role="img"
			aria-label={`${m.dashboard_payment_trend()} ${activeCurrency}`}
		>
			{#each values as entry (entry.month)}
				{@const height = maxAmount === 0 ? 0 : (entry.amount / maxAmount) * 100}
				<div class="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1">
					<div class="flex h-16 w-full items-end justify-center">
						<div
							class="w-full max-w-8 bg-foreground/80 transition-[height]"
							style={`height: ${height}%`}
							title={`${formatMonthLabel(entry.month)}: ${formatCurrency(entry.amount, activeCurrency, currencyDigits)}`}
						></div>
					</div>
					<span class="truncate text-[10px] text-muted-foreground" title={entry.month}>
						{formatMonthLabel(entry.month)}
					</span>
				</div>
			{/each}
		</div>
		<div class="mt-2 flex items-center justify-between gap-3 text-xs">
			<span class="text-muted-foreground">
				{m.dashboard_payments_in_period({
					count: formatNumber(
						values.reduce(
							(total, entry) =>
								total +
								(summary.monthly_payments.find(
									(payment) => payment.month === entry.month && payment.currency_code === activeCurrency
								)?.payment_count ?? 0),
							0
						),
						{ maximumFractionDigits: 0 }
					)
				})}
			</span>
			<strong class="font-medium text-foreground">
				{formatCurrency(values.reduce((total, entry) => total + entry.amount, 0), activeCurrency, currencyDigits)}
			</strong>
		</div>
	{/if}
</Card>
