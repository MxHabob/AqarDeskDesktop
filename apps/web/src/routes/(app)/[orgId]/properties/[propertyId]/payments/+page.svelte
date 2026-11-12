<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { goto } from '$app/navigation';
	import { toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { contractDocumentQueryKeys } from '#lib/features/contracts/queries';
	import PaymentsDirectory from '#lib/features/payments/payments-directory.svelte';
	import { paymentQueryKeys } from '#lib/features/payments/queries';
	import { tenantQueryKeys } from '#lib/features/tenants/queries';
	import { queryClient } from '#lib/core/query-client';
	import * as m from '#lib/paraglide/messages';
	import type { PaymentInput } from '@aqardesk/sdk';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.organizationId);
	const canViewPayments = $derived(
		hasPermission(app.currentUser?.role, 'view_payments', app.currentUser?.permissions)
	);
	const canViewContracts = $derived(
		hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions)
	);
	const canRecord = $derived(
		hasPermission(app.currentUser?.role, 'record_payments', app.currentUser?.permissions)
	);
	const contractsQuery = createQuery(() => ({
		queryKey: contractDocumentQueryKeys.property(organizationId, data.propertyId),
		queryFn: () => desktopApi.listContractDocuments(data.propertyId),
		enabled: !!app.currentUser && canViewPayments && canViewContracts
	}));
	const tenantsQuery = createQuery(() => ({
		queryKey: tenantQueryKeys.all(organizationId),
		queryFn: desktopApi.listTenants,
		enabled: !!app.currentUser && canRecord
	}));
	const paymentsQuery = createQuery(() => ({
		queryKey: paymentQueryKeys.all(organizationId),
		queryFn: desktopApi.listPayments,
		enabled: !!app.currentUser && canViewPayments && canViewContracts
	}));
	const currenciesQuery = createQuery(() => ({
		queryKey: ['currencies', organizationId],
		queryFn: desktopApi.listCurrencies,
		enabled: !!app.currentUser && canViewPayments
	}));
	const contractIds = $derived(
		(contractsQuery.data ?? []).map((contract) => contract.id)
	);
	const contractIdSet = $derived(new Set(contractIds));
	const propertyTenantIds = $derived(
		new Set((contractsQuery.data ?? []).map((contract) => contract.tenant_id))
	);
	const tenants = $derived(
		(tenantsQuery.data ?? []).filter((tenant) => propertyTenantIds.has(tenant.id))
	);
	const payments = $derived(
		(paymentsQuery.data ?? []).filter((payment) =>
			payment.allocations.some((allocation) => contractIdSet.has(allocation.contract_document_id))
		)
	);
	const loading = $derived(
		contractsQuery.isLoading ||
			paymentsQuery.isLoading ||
			currenciesQuery.isLoading ||
			(canRecord && tenantsQuery.isLoading)
	);
	let busy = $state(false);

	$effect(() => {
		if (contractsQuery.error) app.setError(contractsQuery.error);
		if (paymentsQuery.error) app.setError(paymentsQuery.error);
		if (currenciesQuery.error) app.setError(currenciesQuery.error);
		if (canRecord && tenantsQuery.error) app.setError(tenantsQuery.error);
	});

	async function recordPayment(input: PaymentInput) {
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

	function openPayment(paymentId: string) {
		void goto(`/${organizationId}/payments/${paymentId}`);
	}
</script>

<div class="flex min-h-0 min-w-0 flex-1 flex-col">
	<PaymentsDirectory
		{payments}
		{tenants}
		currencies={currenciesQuery.data ?? []}
		{loading}
		{busy}
		{canRecord}
		canVoid={false}
		{contractIds}
		recordDisabledMessage={m.properties_payments_need_contract()}
		emptyDescription={m.properties_payments_empty_description()}
		onRecord={recordPayment}
		onOpen={(payment) => openPayment(payment.id)}
	/>
</div>
