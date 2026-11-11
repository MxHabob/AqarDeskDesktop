<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { toast } from '@aqardesk/ui';
	import type { CurrencySettings, Payment, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import {
		formatCurrency,
		formatDate as formatLocalizedDate,
		formatGridCell
	} from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import PaymentSheet from '#lib/features/payments/payment-sheet.svelte';
	import * as m from '#lib/paraglide/messages';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';

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
	const isReadonly = $derived(app.currentUser?.role === 'readonly');
	const rtl = $derived(locale === 'ar');

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
		return formatCurrency(payment.amount_minor, payment.currency_code, digits);
	}

	function formatDate(value: string) {
		return formatLocalizedDate(value, 'datetime', undefined, app.organizationTimezone);
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

	const gridRows = $derived(
		payments.map((payment) => ({
			...payment,
			received_at_display: formatDate(payment.received_at),
			amount_display: formatMoney(payment),
			method_display: methodLabel(payment.payment_method)
		}))
	);

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'received_at_display', header: m.payments_received_at(), sortable: false, width: 180 },
		{ id: 'tenant_name', header: m.payments_tenant(), width: 180 },
		{
			id: 'amount_display',
			header: m.payments_amount(),
			sortable: false,
			align: 'end',
			cellClass: 'dir-ltr',
			width: 150
		},
		{ id: 'method_display', header: m.payments_method(), sortable: false },
		{
			id: 'status',
			header: m.payments_status(),
			type: 'status',
			options: [
				{ value: 'posted', label: m.payments_status_posted() },
				{ value: 'void', label: m.payments_status_void() }
			]
		},
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'open',
						icon: 'ArrowUpRight',
						label: m.payments_open_receipt(),
						variant: 'outline'
					},
					{
						id: 'void',
						icon: 'Ban',
						label: m.payments_void(),
						variant: 'destructive',
						visible: (row: Record<string, unknown>) => row.status === 'posted',
						disabled: () => busy || isReadonly
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const payment = payments.find((item) => item.id === String(event.row.id ?? ''));
		if (!payment) return;
		if (event.actionId === 'open') void goto(`/${organizationId}/payments/${payment.id}`);
		if (event.actionId === 'void') void voidPayment(payment);
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={gridRows}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		emptyTitle={m.payments_empty_title()}
		emptyDescription={m.payments_empty_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<PaymentSheet
				{tenants}
				currencies={enabledCurrencies}
				{busy}
				disabled={isReadonly || tenants.length === 0}
				onSubmit={record}
			/>
			{#if !loading && tenants.length === 0}
				<span class="px-2 text-xs text-muted-foreground">{m.payments_need_tenant()}</span>
			{/if}
		{/snippet}
	</AqarDeskGrid>
</div>
