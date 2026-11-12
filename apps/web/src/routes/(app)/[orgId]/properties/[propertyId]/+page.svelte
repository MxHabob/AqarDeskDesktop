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
	<div class="grid min-h-0 flex-1 content-start gap-3" role="status">
		<div class="h-32 animate-pulse rounded-xl border border-border bg-card"></div>
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
			{#each [0, 1, 2, 3] as index (index)}
				<Card class="h-[182px] animate-pulse p-3">
					<div class="h-3 w-24 rounded bg-muted"></div>
					<div class="mt-8 h-7 w-16 rounded bg-muted"></div>
				</Card>
			{/each}
		</div>
	</div>
{:else if property}
	<div class="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto scrollbar-none">
		<section class="rounded-xl border border-border bg-card p-4 sm:p-5">
			<header class="flex flex-wrap items-start justify-between gap-4">
				<div class="flex min-w-0 items-start gap-3">
					<span class="grid size-11 shrink-0 place-items-center rounded-xl bg-accent text-accent-foreground">
						<Building2 size={20} />
					</span>
					<div class="min-w-0">
						<h1 class="truncate text-xl font-semibold tracking-tight">{property.name}</h1>
						<p class="mt-1 truncate text-sm text-muted-foreground">
							{propertyTypeLabel(property.property_type)} · {addressOf(property) || property.code}
						</p>
						<div class="mt-2 flex flex-wrap items-center gap-2">
							<span class="rounded-md border border-border px-2.5 py-1 text-xs">
								{statusLabel(property.status)}
							</span>
							<span class="text-xs text-muted-foreground" dir="ltr">{property.code}</span>
						</div>
					</div>
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<Button
						variant="outline"
						disabled={!canUpdateProperty}
						onclick={() => (propertySheetOpen = true)}
					>
						<Pencil size={16} />{m.properties_edit()}
					</Button>
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
					{#if canViewContracts}
						<Button disabled={!canCreateContractForProperty} onclick={createContract}>
							<Plus size={16} />{m.contracts_new_contract()}
						</Button>
					{/if}
				</div>
			</header>
			{#if property.description}
				<p class="mt-4 max-w-4xl whitespace-pre-wrap border-t border-border pt-4 text-sm leading-6 text-muted-foreground">
					{property.description}
				</p>
			{/if}
		</section>

		{#if canViewContracts && propertyDashboardQuery.data}
			<section class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4" aria-label={m.dashboard_statistics_title()}>
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
				{#if canViewUnits}
					<DashboardStatCard label={m.dashboard_units_count()} value={units.length} icon={Building2} />
				{/if}
				<DashboardPaymentsByCurrencyCard
					label={m.property_dashboard_rent_title()}
					totals={propertyDashboardQuery.data.rent_by_currency}
					emptyLabel={m.property_dashboard_no_amounts()}
					icon={ReceiptText}
				/>
				{#if canViewPayments && propertyDashboardQuery.data.posted_allocations_by_currency}
					<DashboardPaymentsByCurrencyCard
						label={m.property_dashboard_paid_title()}
						totals={propertyDashboardQuery.data.posted_allocations_by_currency}
						emptyLabel={m.property_dashboard_no_amounts()}
					/>
				{/if}
			</section>
			{#if canViewPayments}
				<p class="text-xs leading-5 text-muted-foreground">
					{m.property_dashboard_balance_note()}
				</p>
			{/if}
		{/if}

		{#if canViewContracts}
			<section class="rounded-xl border border-border bg-card p-4 sm:p-5">
				<header class="mb-4 flex items-center justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold">{m.property_dashboard_recent_contracts()}</h2>
						<p class="mt-1 text-xs text-muted-foreground">{m.property_dashboard_contracts_hint()}</p>
					</div>
					<a
						href={`/${organizationId}/properties/${data.propertyId}/contracts`}
						class="inline-flex shrink-0 items-center gap-1 text-xs font-medium text-primary hover:underline"
					>
						{m.property_dashboard_view_contracts()}<ArrowUpRight size={14} />
					</a>
				</header>
				{#if contractDocumentsQuery.isLoading}
					<div class="h-20 animate-pulse rounded-lg bg-muted"></div>
				{:else if contractDocuments.length === 0}
					<div class="rounded-lg border border-dashed border-border px-4 py-8 text-center text-sm text-muted-foreground">
						{m.property_dashboard_no_contracts()}
					</div>
				{:else}
					<div class="grid gap-2">
						{#each contractDocuments.slice(0, 5) as contract (contract.id)}
							<div class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border px-3 py-3">
								<div class="min-w-0">
									<p class="truncate text-sm font-medium">{contract.tenant_name}</p>
									<p class="mt-1 text-xs text-muted-foreground">
										<span dir="ltr">{contract.contract_number}</span>
										<span> · {m.nav_units()} {contract.unit_number}</span>
										<span> · {formatDate(contract.start_date, 'date')} – {formatDate(contract.end_date, 'date')}</span>
									</p>
								</div>
								<strong class="shrink-0 text-sm font-medium" dir="auto">
									{formatCurrency(contract.rent_amount_minor, contract.currency_code, contract.currency_decimal_digits)}
								</strong>
							</div>
						{/each}
					</div>
				{/if}
			</section>
		{/if}

		<div class="grid gap-4 lg:grid-cols-2">
			<section class="rounded-xl border border-border bg-card p-4 sm:p-5">
				<h2 class="text-base font-semibold">{m.properties_details()}</h2>
				<dl class="mt-4 grid gap-x-6 gap-y-4 sm:grid-cols-2">
					<div>
						<dt class="text-xs text-muted-foreground">{m.properties_type()}</dt>
						<dd class="mt-1 text-sm">{propertyTypeLabel(property.property_type)}</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{m.properties_lot_area()}</dt>
						<dd class="mt-1 text-sm">{displayNumber(property.lot_area_square_meters)} م²</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{m.properties_building_area()}</dt>
						<dd class="mt-1 text-sm">{displayNumber(property.building_area_square_meters)} م²</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{m.properties_frontage()}</dt>
						<dd class="mt-1 text-sm">{displayNumber(property.frontage_meters)} م</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{m.properties_depth()}</dt>
						<dd class="mt-1 text-sm">{displayNumber(property.depth_meters)} م</dd>
					</div>
					<div>
						<dt class="text-xs text-muted-foreground">{m.settings_created_at()}</dt>
						<dd class="mt-1 text-sm">{formatDate(property.created_at, 'datetime', undefined, app.organizationTimezone)}</dd>
					</div>
				</dl>
			</section>

			{#if canViewOwners && property.owners.length > 0}
				<section class="rounded-xl border border-border bg-card p-4 sm:p-5">
					<h2 class="text-base font-semibold">{m.properties_owners()}</h2>
					<div class="mt-4 grid gap-3 sm:grid-cols-2">
						{#each property.owners as owner (owner.id)}
							<div class="rounded-lg border border-border p-3">
								<p class="text-sm font-medium">{owner.name}</p>
								<p class="mt-1 text-sm text-muted-foreground" dir="ltr">{owner.phone_number}</p>
								{#if owner.ownership_share_percent != null}
									<p class="mt-1 text-xs text-muted-foreground">
										{formatNumber(owner.ownership_share_percent)}%
									</p>
								{/if}
							</div>
						{/each}
					</div>
				</section>
			{/if}
		</div>
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
