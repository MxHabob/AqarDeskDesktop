<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { getLocale } from '#lib/paraglide/runtime';
	import type { DashboardSummary } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();
	let selectedCurrency = $state('');

	const currencies = $derived(
		[...new Set(summary.monthly_payments.map((payment) => payment.currency_code))].sort()
	);
	const activeCurrency = $derived(
		currencies.includes(selectedCurrency) ? selectedCurrency : (currencies[0] ?? '')
	);
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

	function formatAmount(amountMinor: number) {
		if (!activeCurrency) return '';
		return new Intl.NumberFormat(getLocale(), {
			style: 'currency',
			currency: activeCurrency,
			maximumFractionDigits: currencyDigits
		}).format(amountMinor / 10 ** currencyDigits);
	}

	function formatMonth(month: string) {
		return new Intl.DateTimeFormat(getLocale(), { month: 'short' }).format(
			new Date(`${month}-01T00:00:00`)
		);
	}
</script>

<Card class="flex h-[182px] min-w-0 flex-col p-3">
	<div class="mb-2 flex items-center justify-between gap-3">
		<h3 class="text-xs font-medium text-muted-foreground">{m.dashboard_payment_trend()}</h3>
		{#if currencies.length > 0}
			<select
				bind:value={selectedCurrency}
				class="h-7 max-w-24 rounded-sm border border-border/70 bg-background px-2 text-xs text-foreground"
				aria-label={m.dashboard_payment_trend()}
			>
				{#each currencies as currency (currency)}
					<option value={currency}>{currency}</option>
				{/each}
			</select>
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
							title={`${formatMonth(entry.month)}: ${formatAmount(entry.amount)}`}
						></div>
					</div>
					<span class="truncate text-[10px] text-muted-foreground" title={entry.month}>
						{formatMonth(entry.month)}
					</span>
				</div>
			{/each}
		</div>
		<div class="mt-2 flex items-center justify-between gap-3 text-xs">
			<span class="text-muted-foreground">
				{m.dashboard_payments_in_period({
					count: values.reduce(
						(total, entry) =>
							total +
							(summary.monthly_payments.find(
								(payment) => payment.month === entry.month && payment.currency_code === activeCurrency
							)?.payment_count ?? 0),
						0
					)
				})}
			</span>
			<strong class="font-medium text-foreground">
				{formatAmount(values.reduce((total, entry) => total + entry.amount, 0))}
			</strong>
		</div>
	{/if}
</Card>
