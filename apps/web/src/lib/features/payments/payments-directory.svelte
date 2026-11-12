<script lang="ts">
	import type { CurrencySettings, Payment, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatCurrency, formatGridCell } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import PaymentSheet from '#lib/features/payments/payment-sheet.svelte';
	import * as m from '#lib/paraglide/messages';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		payments: Payment[];
		tenants: Tenant[];
		currencies: CurrencySettings[];
		loading: boolean;
		hasMore?: boolean;
		onloadmore?: () => Promise<unknown>;
		busy: boolean;
		canRecord: boolean;
		canVoid: boolean;
		contractIds?: string[];
		recordDisabledMessage?: string;
		emptyDescription?: string;
		onRecord?: (input: PaymentInput) => Promise<boolean>;
		onVoid?: (payment: Payment) => void;
		onOpen: (payment: Payment) => void;
	};

	let {
		payments,
		tenants,
		currencies,
		loading,
		hasMore = false,
		onloadmore,
		busy,
		canRecord,
		canVoid,
		contractIds,
		recordDisabledMessage,
		emptyDescription,
		onRecord,
		onVoid,
		onOpen
	}: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	const enabledCurrencies = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);

	function formatMoney(payment: Payment) {
		const digits =
			currencies.find((currency) => currency.code === payment.currency_code)?.decimal_digits ?? 2;
		return formatCurrency(payment.amount_minor, payment.currency_code, digits);
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
			amount_display: formatMoney(payment),
			allocation_display: payment.allocations
				.map(
					(allocation) =>
						`${allocation.property_name} (${allocation.unit_number}) · ${formatCurrency(
							allocation.amount_minor,
							payment.currency_code,
							currencies.find((currency) => currency.code === payment.currency_code)?.decimal_digits ?? 2
						)}`
				)
				.join(', ') || '—',
			method_display: methodLabel(payment.payment_method)
		}))
	);

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'received_at', header: m.payments_received_at(), type: 'datetime', width: 180 },
		{ id: 'tenant_name', header: m.payments_tenant(), width: 180 },
		{ id: 'tenant_id', header: m.payments_tenant_id(), width: 220 },
		{ id: 'allocation_display', header: m.payments_allocation_column(), width: 260 },
		{
			id: 'amount_display',
			header: m.payments_amount(),
			align: 'end',
			cellClass: 'dir-ltr',
			width: 150
		},
		{ id: 'currency_code', header: m.payments_currency(), width: 120 },
		{ id: 'payment_method', header: m.payments_method(), width: 150 },
		{ id: 'external_reference', header: m.payments_reference(), width: 180 },
		{ id: 'notes', header: m.common_notes(), width: 240 },
		{
			id: 'status',
			header: m.payments_status(),
			type: 'status',
			options: [
				{ value: 'posted', label: m.payments_status_posted() },
				{ value: 'void', label: m.payments_status_void() }
			]
		},
		{ id: 'voided_at', header: m.payments_voided_at(), type: 'datetime', width: 180 },
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
						visible: (row: Record<string, unknown>) =>
							row.status === 'posted' && canVoid && !!onVoid,
						disabled: () => busy || !canVoid || !onVoid
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const payment = payments.find((item) => item.id === String(event.row.id ?? ''));
		if (!payment) return;
		if (event.actionId === 'dblclick' || event.actionId === 'open') onOpen(payment);
		if (event.actionId === 'void') onVoid?.(payment);
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
		{hasMore}
		{onloadmore}
		emptyTitle={m.payments_empty_title()}
		emptyDescription={emptyDescription ?? m.payments_empty_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			{#if onRecord}
				<PaymentSheet
					{tenants}
					currencies={enabledCurrencies}
					{busy}
					{contractIds}
					disabled={!canRecord || tenants.length === 0}
					onSubmit={onRecord}
				/>
				{#if canRecord && !loading && tenants.length === 0}
					<span class="px-2 text-xs text-muted-foreground">
						{recordDisabledMessage ?? m.payments_need_tenant()}
					</span>
				{/if}
			{/if}
		{/snippet}
	</AqarDeskGrid>
</div>
