<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { ArrowUpRight, Ban, ReceiptText } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle, toast } from '@aqardesk/ui';
	import type { CurrencySettings, Payment, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import PaymentSheet from '#lib/features/payments/payment-sheet.svelte';
	import EmptyState from '#lib/shared/components/empty-state.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const locale = $derived(getLocale());
	const organizationId = $derived(data.orgId);
	let payments = $state<Payment[]>([]);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
	let loading = $state(true);
	let busy = $state(false);
	const enabledCurrencies = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			[payments, tenants, currencies] = await Promise.all([
				desktopApi.listPayments(),
				desktopApi.listTenants(),
				desktopApi.listCurrencies()
			]);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function record(input: PaymentInput) {
		busy = true;
		try {
			const payment = await desktopApi.recordPayment(input);
			payments = [payment, ...payments];
			toast.success(m.payments_recorded());
			return true;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
			return false;
		} finally {
			busy = false;
		}
	}

	async function voidPayment(payment: Payment) {
		const confirmed = await confirmStore.prompt({
			title: m.payments_void_confirm_title(),
			description: m.payments_void_confirm_description({ name: payment.tenant_name }),
			confirmText: m.payments_void(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;
		busy = true;
		try {
			await desktopApi.voidPayment(payment.id);
			payments = payments.map((item) =>
				item.id === payment.id
					? { ...item, status: 'void', voided_at: new Date().toISOString() }
					: item
			);
			toast.success(m.payments_voided());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			busy = false;
		}
	}

	function formatMoney(payment: Payment) {
		const digits =
			currencies.find((currency) => currency.code === payment.currency_code)?.decimal_digits ?? 2;
		return new Intl.NumberFormat(locale, {
			style: 'currency',
			currency: payment.currency_code,
			minimumFractionDigits: digits,
			maximumFractionDigits: digits
		}).format(payment.amount_minor / 10 ** digits);
	}

	function formatDate(value: string) {
		return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(
			new Date(value)
		);
	}

	function methodLabel(method: string) {
		switch (method) {
			case 'cash': return m.payments_method_cash();
			case 'bank_transfer': return m.payments_method_bank_transfer();
			case 'card': return m.payments_method_card();
			case 'cheque': return m.payments_method_cheque();
			case 'online': return m.payments_method_online();
			default: return m.payments_method_other();
		}
	}
</script>

<header class="flex flex-wrap items-start justify-between gap-3">
	<div class="grid gap-1">
		<h2 class="text-lg font-semibold">{m.nav_payments()}</h2>
		<p class="text-sm text-muted-foreground">{m.dashboard_payments_description()}</p>
	</div>
	<PaymentSheet
		{tenants}
		currencies={enabledCurrencies}
		{busy}
		disabled={app.currentUser?.role === 'readonly'}
		onSubmit={record}
	/>
	{#if !loading && tenants.length === 0}
		<p class="w-full text-xs text-muted-foreground">{m.payments_need_tenant()}</p>
	{/if}
</header>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1">
			<CardTitle class="text-base">{m.payments_receipts()}</CardTitle>
			<CardDescription>{payments.length} {m.payments_receipts()}</CardDescription>
		</div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground">
			<ReceiptText size={18} />
		</span>
	</CardHeader>
	{#if loading}
		<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
	{:else if payments.length === 0}
		<EmptyState title={m.payments_empty_title()} description={m.payments_empty_description()} />
	{:else}
		<div class="overflow-x-auto scrollbar-none">
			<table class="w-full min-w-[54rem] border-collapse text-start text-sm">
				<thead class="border-b border-border text-xs font-medium text-muted-foreground">
					<tr>
						<th class="px-2 py-3 text-start">{m.payments_received_at()}</th>
						<th class="px-2 py-3 text-start">{m.payments_tenant()}</th>
						<th class="px-2 py-3 text-start">{m.payments_amount()}</th>
						<th class="px-2 py-3 text-start">{m.payments_method()}</th>
						<th class="px-2 py-3 text-start">{m.payments_status()}</th>
						<th class="px-2 py-3 text-start">{m.users_action()}</th>
					</tr>
				</thead>
				<tbody>
					{#each payments as payment (payment.id)}
						<tr class="border-b border-border/60 last:border-0">
							<td class="px-2 py-3">{formatDate(payment.received_at)}</td>
							<td class="px-2 py-3 font-medium">{payment.tenant_name}</td>
							<td class="px-2 py-3 font-medium" dir="ltr">{formatMoney(payment)}</td>
							<td class="px-2 py-3 text-muted-foreground">{methodLabel(payment.payment_method)}</td>
							<td class="px-2 py-3">
								<span class={payment.status === 'posted' ? 'text-emerald-700 dark:text-emerald-400' : 'text-destructive'}>
									{payment.status === 'posted' ? m.payments_status_posted() : m.payments_status_void()}
								</span>
							</td>
							<td class="px-2 py-3">
								<div class="flex gap-2">
									<Button
										variant="outline"
										size="icon"
										class="size-8"
										aria-label={m.payments_open_receipt()}
										onclick={() => void goto(`/${organizationId}/payments/${payment.id}`)}
									>
										<ArrowUpRight size={15} />
									</Button>
									{#if payment.status === 'posted'}
										<Button
											variant="outline"
											size="icon"
											class="size-8 text-destructive"
											disabled={busy || app.currentUser?.role === 'readonly'}
											aria-label={m.payments_void()}
											onclick={() => void voidPayment(payment)}
										>
											<Ban size={15} />
										</Button>
									{/if}
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</Card>
