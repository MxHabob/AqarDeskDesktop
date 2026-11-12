<script lang="ts">
	import { goto } from '$app/navigation';
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import { Button, Card } from '@aqardesk/ui';
	import { ArrowUpRight, Building2, FileText, Pencil, Plus, ReceiptText, UsersRound } from '@lucide/svelte';
	import type { PageProps } from './$types';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatCurrency, formatDate, formatNumber } from '#lib/core/i18n';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { dashboardQueries, dashboardQueryKeys } from '#lib/features/dashboard/dashboard-queries';
	import DashboardPaymentsByCurrencyCard from '#lib/features/dashboard/dashboard-payments-by-currency-card.svelte';
	import DashboardStatCard from '#lib/features/dashboard/dashboard-stat-card.svelte';
	import { contractDocumentQueryKeys } from '#lib/features/contracts/queries';
	import PropertySheet from '#lib/features/properties/property-sheet.svelte';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import { ownerQueryKeys } from '#lib/features/owners/queries';
	import TenantSheet from '#lib/features/tenants/tenant-sheet.svelte';
	import { tenantQueryKeys } from '#lib/features/tenants/queries';
	import UnitSheet from '#lib/features/units/unit-sheet.svelte';
	import { unitQueryKeys } from '#lib/features/units/queries';
	import { queryClient } from '#lib/core/query-client';
	import * as m from '#lib/paraglide/messages';
	import type { Property, PropertyInput, PropertyUnitInput, TenantInput } from '@aqardesk/sdk';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.organizationId);
	const canViewUnits = $derived(
		hasPermission(app.currentUser?.role, 'view_units', app.currentUser?.permissions)
	);
	const canViewContracts = $derived(
		hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions)
	);
	const canViewPayments = $derived(
		hasPermission(app.currentUser?.role, 'view_payments', app.currentUser?.permissions)
	);
	const canViewOwners = $derived(
		hasPermission(app.currentUser?.role, 'view_owners', app.currentUser?.permissions)
	);
	const canUpdateProperty = $derived(
		hasPermission(app.currentUser?.role, 'update_properties', app.currentUser?.permissions)
	);
	const canCreateUnits = $derived(
		hasPermission(app.currentUser?.role, 'create_units', app.currentUser?.permissions)
	);
	const canCreateTenants = $derived(
		hasPermission(app.currentUser?.role, 'create_tenants', app.currentUser?.permissions)
	);
	const canCreateContracts = $derived(
		hasPermission(app.currentUser?.role, 'create_contracts', app.currentUser?.permissions)
	);
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser
	}));
	const unitsQuery = createQuery(() => ({
		queryKey: unitQueryKeys.all(organizationId),
		queryFn: desktopApi.listUnits,
		enabled: !!app.currentUser && canViewUnits
	}));
	let propertySheetOpen = $state(false);
	let savingProperty = $state(false);
	const ownersQuery = createQuery(() => ({
		queryKey: ownerQueryKeys.all(organizationId),
		queryFn: desktopApi.listOwners,
		enabled: !!app.currentUser && canViewOwners && propertySheetOpen
	}));
	const pageSize = 50;
	const contractDocumentsQuery = createInfiniteQuery(() => ({
		queryKey: contractDocumentQueryKeys.pages(organizationId, data.propertyId),
		queryFn: ({ pageParam }) =>
			desktopApi.listContractDocumentsPage(pageSize, pageParam, data.propertyId),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser && canViewContracts
	}));
	const propertyDashboardQuery = createQuery(() => ({
		queryKey: dashboardQueryKeys.propertySummary(organizationId, data.propertyId),
		queryFn: () => dashboardQueries.propertySummary(data.propertyId),
		enabled: !!app.currentUser && canViewContracts
	}));
	const property = $derived(
		(propertiesQuery.data ?? []).find((item) => item.id === data.propertyId)
	);
	const units = $derived(
		canViewUnits
			? (unitsQuery.data ?? []).filter((item) => item.property_id === data.propertyId)
			: []
	);
	const canCreateContractForProperty = $derived(
		canCreateContracts &&
			property?.status === 'active' &&
			units.some((unit) => unit.status !== 'inactive')
	);
	const loading = $derived(propertiesQuery.isLoading || (canViewUnits && unitsQuery.isLoading));
	const owners = $derived(ownersQuery.data ?? []);
	const contractDocuments = $derived(
		contractDocumentsQuery.data?.pages.flatMap((page) => page.items) ?? []
	);
	let unitSheetOpen = $state(false);
	let tenantSheetOpen = $state(false);
	let savingUnit = $state(false);
	let savingTenant = $state(false);

	$effect(() => {
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
		if (canViewUnits && unitsQuery.error) app.setError(unitsQuery.error);
		if (canViewContracts && contractDocumentsQuery.error) app.setError(contractDocumentsQuery.error);
		if (canViewContracts && propertyDashboardQuery.error) app.setError(propertyDashboardQuery.error);
		if (canViewOwners && propertySheetOpen && ownersQuery.error) app.setError(ownersQuery.error);
	});

	async function saveProperty(values: PropertyInput) {
		if (!property) {
			app.setError(new Error('The property is not available for editing.'));
			return false;
		}
		savingProperty = true;
		try {
			await desktopApi.updateProperty(property.id, values);
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) }),
				queryClient.invalidateQueries({ queryKey: ownerQueryKeys.all(organizationId) })
			]);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingProperty = false;
		}
	}

	async function saveUnit(values: PropertyUnitInput) {
		savingUnit = true;
		try {
			await desktopApi.createUnit(values);
			await queryClient.invalidateQueries({ queryKey: unitQueryKeys.all(organizationId) });
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingUnit = false;
		}
	}

	async function saveTenant(values: TenantInput) {
		savingTenant = true;
		try {
			const tenant = await desktopApi.createTenant(values);
			await queryClient.invalidateQueries({ queryKey: tenantQueryKeys.all(organizationId) });
			if (canCreateContracts) {
				void goto(
					`/${organizationId}/properties/${data.propertyId}/contracts?tenantId=${encodeURIComponent(tenant.id)}&create=1`
				);
			}
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingTenant = false;
		}
	}

	function createContract() {
		void goto(
			`/${organizationId}/properties/${data.propertyId}/contracts?create=1`
		);
	}

	function propertyTypeLabel(value: Property['property_type']) {
		switch (value) {
			case 'residential': return m.properties_type_residential();
			case 'commercial': return m.properties_type_commercial();
			case 'mixed_use': return m.properties_type_mixed();
			case 'land': return m.properties_type_land();
			case 'building': return m.properties_type_building();
			case 'house_villa': return m.properties_type_house_villa();
			case 'investment': return m.properties_type_investment();
			case 'warehouse': return m.properties_type_warehouse();
			case 'yard': return m.properties_type_yard();
			case 'apartment': return m.properties_type_apartment();
			default: return m.common_other();
		}
	}

	function addressOf(value: Property) {
		return [
			value.address_line_1,
			value.address_line_2,
			value.neighborhood,
			value.district,
			value.city,
			value.region,
			value.postal_code,
			value.country_code
		]
			.filter(Boolean)
			.join('، ');
	}

	function statusLabel(value: Property['status']) {
		return value === 'active' ? m.common_enabled() : m.common_disabled();
	}

	function displayNumber(value: number | null | undefined) {
		return value == null ? '—' : formatNumber(value);
	}
</script>

{#if loading}
	<div class="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden" role="status">
		<div class="h-10 w-1/3 animate-pulse rounded bg-muted"></div>
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			{#each [0, 1, 2, 3] as index (index)}
				<Card class="h-[182px] animate-pulse p-3">
					<div class="h-3 w-24 rounded bg-muted"></div>
					<div class="mt-8 h-7 w-16 rounded bg-muted"></div>
				</Card>
			{/each}
		</div>
		<div class="grid min-w-0 shrink-0 grid-cols-1 gap-3 lg:grid-cols-4">
			<Card class="h-[182px] animate-pulse p-3 lg:col-span-2"></Card>
			<Card class="h-[182px] animate-pulse p-3"></Card>
			<Card class="h-[182px] animate-pulse p-3"></Card>
		</div>
	</div>
{:else if property}
	<div class="flex min-h-0 min-w-0 flex-1 flex-col gap-3 overflow-hidden">
		<header class="flex min-h-10 flex-wrap items-start justify-between gap-3 sm:flex-nowrap sm:items-center">
			<div class="grid min-w-0 gap-1">
				<h1 class="truncate font-serif text-3xl font-normal tracking-tight text-foreground">
					{property.name}
				</h1>
				<p class="truncate text-xs text-muted-foreground">
					{addressOf(property) || property.code}
					<span aria-hidden="true"> · </span>
					{propertyTypeLabel(property.property_type)}
					<span aria-hidden="true"> · </span>
					<span dir="ltr">{property.code}</span>
					<span aria-hidden="true"> · </span>
					{statusLabel(property.status)}
				</p>
			</div>
			<div class="flex shrink-0 flex-wrap items-center gap-2">
				{#if canViewContracts}
					<Button disabled={!canCreateContractForProperty} onclick={createContract}>
						<Plus size={16} />{m.contracts_new_contract()}
					</Button>
				{/if}
				{#if canViewUnits}
					<Button
						variant="outline"
						disabled={!canCreateUnits}
						onclick={() => (unitSheetOpen = true)}
					>
						<Plus size={16} />{m.units_add()}
					</Button>
				{/if}
				<Button
					variant="outline"
					disabled={!canCreateTenants}
					onclick={() => (tenantSheetOpen = true)}
				>
					<Plus size={16} />{m.tenants_add()}
				</Button>
				<Button
					variant="outline"
					disabled={!canUpdateProperty}
					onclick={() => (propertySheetOpen = true)}
				>
					<Pencil size={16} />{m.properties_edit()}
				</Button>
			</div>
		</header>

		<section class="grid grid-cols-2 gap-3 lg:grid-cols-4" aria-label={m.dashboard_statistics_title()}>
			{#if canViewContracts && propertyDashboardQuery.data}
				<DashboardStatCard
					label={m.dashboard_contract_count()}
					value={propertyDashboardQuery.data.contract_count}
					icon={FileText}
				/>
				<DashboardStatCard
					label={m.dashboard_tenants_count()}
					value={propertyDashboardQuery.data.tenant_count}
					icon={UsersRound}
				/>
			{/if}
			{#if canViewUnits}
				<DashboardStatCard label={m.dashboard_units_count()} value={units.length} icon={Building2} />
			{/if}
			{#if canViewContracts}
				<DashboardPaymentsByCurrencyCard
					label={m.property_dashboard_rent_title()}
					totals={propertyDashboardQuery.data?.rent_by_currency}
					emptyLabel={m.property_dashboard_no_amounts()}
					icon={ReceiptText}
				/>
			{/if}
		</section>

		<div class="grid min-h-0 flex-1 grid-cols-1 gap-3 lg:grid-cols-4">
			{#if canViewContracts}
				<Card class="flex h-[182px] min-h-0 min-w-0 flex-col p-3 lg:col-span-2">
					<header class="mb-2 flex items-center justify-between gap-3">
						<div class="min-w-0">
							<h2 class="truncate text-xs font-medium text-muted-foreground">
								{m.property_dashboard_recent_contracts()}
							</h2>
							<p class="mt-1 truncate text-[11px] text-muted-foreground">
								{m.property_dashboard_contracts_hint()}
							</p>
						</div>
						<a
							href={`/${organizationId}/properties/${data.propertyId}/contracts`}
							class="inline-flex shrink-0 items-center gap-1 text-xs font-medium text-primary hover:underline"
						>
							{m.property_dashboard_view_contracts()}<ArrowUpRight size={14} />
						</a>
					</header>
					{#if contractDocumentsQuery.isLoading}
						<div class="grid min-h-0 flex-1 content-center gap-2" role="status" aria-label={m.common_loading()}>
							{#each [0, 1, 2] as index (index)}
								<div class="h-7 animate-pulse rounded bg-muted"></div>
							{/each}
						</div>
					{:else if contractDocuments.length === 0}
						<div class="grid min-h-0 flex-1 place-items-center rounded border border-dashed border-border px-3 text-center text-xs text-muted-foreground">
							{m.property_dashboard_no_contracts()}
						</div>
					{:else}
						<div class="min-h-0 flex-1 divide-y divide-border overflow-hidden">
							{#each contractDocuments.slice(0, 3) as contract (contract.id)}
								<div class="flex min-w-0 items-center justify-between gap-3 py-2">
									<div class="min-w-0">
										<p class="truncate text-xs font-medium">{contract.tenant_name}</p>
										<p class="mt-1 truncate text-[10px] text-muted-foreground">
											<span dir="ltr">{contract.contract_number}</span>
											<span> · {m.nav_units()} {contract.unit_number}</span>
											<span> · {formatDate(contract.end_date, 'date')}</span>
										</p>
									</div>
									<strong class="shrink-0 text-xs font-medium" dir="auto">
										{formatCurrency(contract.rent_amount_minor, contract.currency_code, contract.currency_decimal_digits)}
									</strong>
								</div>
							{/each}
						</div>
					{/if}
				</Card>
			{/if}

			<Card class="flex h-[182px] min-w-0 flex-col p-3">
				<h2 class="mb-2 text-xs font-medium text-muted-foreground">{m.properties_details()}</h2>
				<dl class="grid grid-cols-2 gap-x-3 gap-y-2 text-xs">
					<div>
						<dt class="text-[10px] text-muted-foreground">{m.properties_type()}</dt>
						<dd class="mt-0.5 truncate">{propertyTypeLabel(property.property_type)}</dd>
					</div>
					<div>
						<dt class="text-[10px] text-muted-foreground">{m.properties_lot_area()}</dt>
						<dd class="mt-0.5 truncate">{displayNumber(property.lot_area_square_meters)} m²</dd>
					</div>
					<div>
						<dt class="text-[10px] text-muted-foreground">{m.properties_building_area()}</dt>
						<dd class="mt-0.5 truncate">{displayNumber(property.building_area_square_meters)} m²</dd>
					</div>
					<div>
						<dt class="text-[10px] text-muted-foreground">{m.settings_created_at()}</dt>
						<dd class="mt-0.5 truncate">{formatDate(property.created_at, 'date', undefined, app.organizationTimezone)}</dd>
					</div>
				</dl>
			</Card>

			{#if canViewPayments && propertyDashboardQuery.data?.posted_allocations_by_currency}
				<DashboardPaymentsByCurrencyCard
					label={m.property_dashboard_paid_title()}
					totals={propertyDashboardQuery.data.posted_allocations_by_currency}
					emptyLabel={m.property_dashboard_no_amounts()}
				/>
			{:else if canViewPayments}
				<Card class="flex h-[182px] min-w-0 flex-col p-3">
					<h2 class="mb-2 text-xs font-medium text-muted-foreground">{m.property_dashboard_paid_title()}</h2>
					<p class="grid flex-1 place-items-center text-xs text-muted-foreground">
						{m.property_dashboard_no_amounts()}
					</p>
				</Card>
			{/if}
		</div>

		{#if canViewOwners}
			<Card class="grid min-h-[182px] shrink-0 content-start gap-3 p-3">
				<header class="flex items-center justify-between gap-3">
					<div class="min-w-0">
						<h2 class="truncate text-xs font-medium text-muted-foreground">{m.properties_owners()}</h2>
						<p class="mt-1 text-[10px] text-muted-foreground">{formatNumber(property.owners.length)}</p>
					</div>
					<UsersRound size={16} class="shrink-0 text-muted-foreground" />
				</header>
				{#if property.owners.length === 0}
					<div class="grid min-h-20 place-items-center rounded border border-dashed border-border px-3 text-center text-xs text-muted-foreground">
						{m.owners_empty()}
					</div>
				{:else}
					<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-4">
						{#each property.owners as owner (owner.id)}
							<div class="flex min-w-0 items-center justify-between gap-2 rounded border border-border/70 px-2.5 py-2">
								<div class="min-w-0">
									<p class="truncate text-xs font-medium">{owner.name}</p>
									<p class="mt-0.5 truncate text-[10px] text-muted-foreground" dir="ltr">
										{owner.phone_number}
									</p>
								</div>
								{#if owner.ownership_share_percent != null}
									<span class="shrink-0 text-[10px] text-muted-foreground">
										{formatNumber(owner.ownership_share_percent)}%
									</span>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</Card>
		{/if}

		{#if canViewPayments}
			<p class="shrink-0 text-[11px] leading-4 text-muted-foreground">
				{m.property_dashboard_balance_note()}
			</p>
		{/if}
	</div>
{:else}
	<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
		{m.properties_not_found()}
	</div>
{/if}

{#if property}
	<PropertySheet
		bind:open={propertySheetOpen}
		initial={property}
		ownerOptions={owners}
		busy={savingProperty}
		onsave={saveProperty}
		onclose={() => (propertySheetOpen = false)}
	/>
	<UnitSheet
		bind:open={unitSheetOpen}
		properties={[property]}
		busy={savingUnit}
		onsave={saveUnit}
		onclose={() => (unitSheetOpen = false)}
	/>
{/if}
<TenantSheet
	bind:open={tenantSheetOpen}
	busy={savingTenant}
	onsave={saveTenant}
	onclose={() => (tenantSheetOpen = false)}
/>
