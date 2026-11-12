<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { toast } from '@aqardesk/ui';
	import type { CurrencySettings, Payment, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import PaymentsDirectory from '#lib/features/payments/payments-directory.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	let payments = $state<Payment[]>([]);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
	let loading = $state(true);
	let busy = $state(false);
	const canView = $derived(
		hasPermission(app.currentUser?.role, 'view_payments', app.currentUser?.permissions)
	);
	const canRecord = $derived(
		hasPermission(app.currentUser?.role, 'record_payments', app.currentUser?.permissions)
	);
	const canVoid = $derived(
		hasPermission(app.currentUser?.role, 'void_payments', app.currentUser?.permissions)
	);

	onMount(() => {
		if (canView) void load();
	});

	async function load() {
		loading = true;
		try {
			[payments, tenants, currencies] = await Promise.all([
				desktopApi.listPayments(),
				canRecord ? desktopApi.listTenants() : Promise.resolve([]),
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

	function openPayment(payment: Payment) {
		void goto(`/${organizationId}/payments/${payment.id}`);
	}
</script>

{#if app.currentUser && canView}
	<div class="min-h-0 min-w-0 flex-1">
		<PaymentsDirectory
			{payments}
			{tenants}
			{currencies}
			{loading}
			{busy}
			{canRecord}
			{canVoid}
			onRecord={record}
			onVoid={(payment) => void voidPayment(payment)}
			onOpen={openPayment}
		/>
	</div>
{/if}
