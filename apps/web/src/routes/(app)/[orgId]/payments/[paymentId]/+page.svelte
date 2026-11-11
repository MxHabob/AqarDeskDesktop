<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { ArrowLeft, Printer, ReceiptText } from '@lucide/svelte';
	import { Button, Card, CardContent, toast } from '@aqardesk/ui';
	import type { CurrencySettings, OrganizationSettings, Payment } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const locale = $derived(getLocale());
	let payment = $state<Payment | null>(null);
	let organization = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let loading = $state(true);

	onMount(() => {
		void load();
	});

	async function load() {
		try {
			const [payments, currentOrganization, availableCurrencies] = await Promise.all([
				desktopApi.listPayments(),
				desktopApi.getOrganizationSettings(),
				desktopApi.listCurrencies()
			]);
			organization = currentOrganization;
			currencies = availableCurrencies;
			payment = payments.find((item) => item.id === data.paymentId) ?? null;
			if (!payment) toast.error(m.payments_not_found());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	function formatMoney(item: Payment) {
		const digits =
			currencies.find((currency) => currency.code === item.currency_code)?.decimal_digits ?? 2;
		return new Intl.NumberFormat(locale, {
			style: 'currency',
			currency: item.currency_code,
			minimumFractionDigits: digits,
			maximumFractionDigits: digits
		}).format(item.amount_minor / 10 ** digits);
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

<svelte:head>
	<title>{m.payments_receipt()} · {organization?.name ?? m.app_name()}</title>
</svelte:head>

<div class="flex items-center justify-between gap-3 print:hidden">
	<Button variant="ghost" class="rounded-md" onclick={() => void goto(`/${data.orgId}/payments`)}>
		<ArrowLeft size={16} />
		{m.payments_back_to_list()}
	</Button>
	{#if payment}
		<Button variant="outline" class="rounded-md" onclick={() => window.print()}>
			<Printer size={16} />
			{m.payments_print_receipt()}
		</Button>
	{/if}
</div>

{#if loading}
	<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
{:else if payment}
	<Card class="mx-auto w-full max-w-2xl rounded-lg bg-card p-5 shadow-card ring-border/50 sm:p-8 print:max-w-none print:border-0 print:shadow-none">
		<CardContent class="grid gap-6 p-0">
			<header class="flex items-start justify-between gap-4 border-b border-border pb-5">
				<div class="grid gap-1">
					<p class="text-xs font-medium text-muted-foreground">{m.payments_receipt()}</p>
					<h2 class="text-xl font-semibold">{organization?.name ?? m.app_name()}</h2>
				</div>
				<span class="grid size-10 place-items-center rounded-lg bg-accent text-accent-foreground print:hidden">
					<ReceiptText size={20} />
				</span>
			</header>
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="grid gap-1">
					<p class="text-xs text-muted-foreground">{m.payments_receipt_number()}</p>
					<p class="break-all font-mono text-sm" dir="ltr">{payment.id}</p>
				</div>
				<div class="grid gap-1">
					<p class="text-xs text-muted-foreground">{m.payments_received_at()}</p>
					<p class="text-sm">{new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(payment.received_at))}</p>
				</div>
				<div class="grid gap-1">
					<p class="text-xs text-muted-foreground">{m.payments_tenant()}</p>
					<p class="text-sm font-medium">{payment.tenant_name}</p>
				</div>
				<div class="grid gap-1">
					<p class="text-xs text-muted-foreground">{m.payments_method()}</p>
					<p class="text-sm">{methodLabel(payment.payment_method)}</p>
				</div>
				{#if payment.external_reference}
					<div class="grid gap-1">
						<p class="text-xs text-muted-foreground">{m.payments_reference()}</p>
						<p class="text-sm" dir="ltr">{payment.external_reference}</p>
					</div>
				{/if}
			</div>
			<div class="grid gap-2 rounded-lg bg-muted/50 p-4">
				<p class="text-xs text-muted-foreground">{m.payments_amount()}</p>
				<p class="text-2xl font-semibold" dir="ltr">{formatMoney(payment)}</p>
			</div>
			{#if payment.notes}
				<div class="grid gap-1">
					<p class="text-xs text-muted-foreground">{m.common_notes()}</p>
					<p class="whitespace-pre-wrap text-sm">{payment.notes}</p>
				</div>
			{/if}
			<p class="border-t border-border pt-4 text-xs text-muted-foreground">
				{payment.status === 'posted' ? m.payments_status_posted() : m.payments_status_void()}
			</p>
		</CardContent>
	</Card>
{/if}

<style>
	@media print {
		:global(body) {
			background: white !important;
		}
	}
</style>
