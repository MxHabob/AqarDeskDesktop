<script lang="ts">
	import { onMount } from 'svelte';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import type { PageProps } from './$types';
	import { toast } from '@aqardesk/ui';
	import type { CurrencySettings, Payment, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { queryClient } from '#lib/core/query-client';
	import { paymentQueryKeys } from '#lib/features/payments/queries';
	import PaymentsDirectory from '#lib/features/payments/payments-directory.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
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
	const pageSize = 50;
	const paymentsQuery = createInfiniteQuery(() => ({
		queryKey: paymentQueryKeys.pages(organizationId),
		queryFn: ({ pageParam }) => desktopApi.listPaymentsPage(pageSize, pageParam),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser && canView
	}));
	const payments = $derived(paymentsQuery.data?.pages.flatMap((page) => page.items) ?? []);
	const loading = $derived(paymentsQuery.isLoading);

	onMount(() => {
		if (canView) void loadAuxiliaryData();
	});

	async function loadAuxiliaryData() {
		try {
			[tenants, currencies] = await Promise.all([
				canRecord ? desktopApi.listTenants() : Promise.resolve([]),
				desktopApi.listCurrencies()
			]);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}

	async function record(input: PaymentInput) {
		busy = true;
		try {
			await desktopApi.recordPayment(input);
			await queryClient.invalidateQueries({ queryKey: paymentQueryKeys.all(organizationId) });
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
			await queryClient.invalidateQueries({ queryKey: paymentQueryKeys.all(organizationId) });
			toast.success(m.payments_voided());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			busy = false;
		}
	}

</script>

{#if app.currentUser && canView}
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">
		<PaymentsDirectory
			{payments}
			{tenants}
			{currencies}
			{loading}
			hasMore={paymentsQuery.hasNextPage}
			onloadmore={() => fetchNextPageOrThrow(() => paymentsQuery.fetchNextPage())}
			{busy}
			{canRecord}
			{canVoid}
			onRecord={record}
			onVoid={(payment) => void voidPayment(payment)}
		/>
	</div>
{/if}
