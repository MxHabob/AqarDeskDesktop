<script lang="ts">
	import { goto } from '$app/navigation';
	import { createQuery } from '@tanstack/svelte-query';
	import { Button } from '@aqardesk/ui';
	import { Plus } from '@lucide/svelte';
	import type { PageProps } from './$types';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatDate, formatNumber } from '#lib/core/i18n';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import TenantSheet from '#lib/features/tenants/tenant-sheet.svelte';
	import { tenantQueryKeys } from '#lib/features/tenants/queries';
	import UnitSheet from '#lib/features/units/unit-sheet.svelte';
	import { unitQueryKeys } from '#lib/features/units/queries';
	import { queryClient } from '#lib/core/query-client';
	import * as m from '#lib/paraglide/messages';
	import type { Property, PropertyUnitInput, TenantInput } from '@aqardesk/sdk';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.organizationId);
	const canViewUnits = $derived(
		hasPermission(app.currentUser?.role, 'view_units', app.currentUser?.permissions)
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
	let unitSheetOpen = $state(false);
	let tenantSheetOpen = $state(false);
	let savingUnit = $state(false);
	let savingTenant = $state(false);

	$effect(() => {
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
		if (canViewUnits && unitsQuery.error) app.setError(unitsQuery.error);
	});

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
	<div class="rounded-xl border border-border bg-card p-5 text-sm text-muted-foreground" role="status">
		{m.common_loading()}
	</div>
{:else if property}
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto scrollbar-none">
		<section class="rounded-xl border border-border bg-card p-5">
			<header class="flex flex-wrap items-start justify-between gap-4">
				<div class="min-w-0">
					<h1 class="truncate text-xl font-semibold">{property.name}</h1>
					<p class="mt-1 text-sm text-muted-foreground" dir="ltr">{property.code}</p>
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<Button
						variant="outline"
						disabled={!canCreateUnits}
						onclick={() => (unitSheetOpen = true)}
					>
						<Plus size={16} />{m.units_add()}
					</Button>
					<Button
						variant="outline"
						disabled={!canCreateTenants}
						onclick={() => (tenantSheetOpen = true)}
					>
						<Plus size={16} />{m.tenants_add()}
					</Button>
					<Button disabled={!canCreateContractForProperty} onclick={createContract}>
						<Plus size={16} />{m.contracts_new_contract()}
					</Button>
					<span class="rounded-md border border-border px-2.5 py-1 text-xs">
						{statusLabel(property.status)}
					</span>
				</div>
			</header>
			<dl class="mt-5 grid gap-x-6 gap-y-4 sm:grid-cols-2 lg:grid-cols-3">
				<div>
					<dt class="text-xs text-muted-foreground">{m.properties_type()}</dt>
					<dd class="mt-1 text-sm">{propertyTypeLabel(property.property_type)}</dd>
				</div>
				<div class="sm:col-span-2">
					<dt class="text-xs text-muted-foreground">{m.properties_address()}</dt>
					<dd class="mt-1 text-sm">{addressOf(property) || '—'}</dd>
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
					<dt class="text-xs text-muted-foreground">{m.properties_latitude()}</dt>
					<dd class="mt-1 text-sm" dir="ltr">{displayNumber(property.latitude)}</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">{m.properties_longitude()}</dt>
					<dd class="mt-1 text-sm" dir="ltr">{displayNumber(property.longitude)}</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">{m.settings_created_at()}</dt>
					<dd class="mt-1 text-sm">{formatDate(property.created_at, 'datetime', undefined, app.organizationTimezone)}</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">{m.settings_updated_at()}</dt>
					<dd class="mt-1 text-sm">{formatDate(property.updated_at, 'datetime', undefined, app.organizationTimezone)}</dd>
				</div>
				{#if canViewUnits}
					<div>
						<dt class="text-xs text-muted-foreground">{m.nav_units()}</dt>
						<dd class="mt-1 text-sm">{units.length}</dd>
					</div>
				{/if}
			</dl>
			{#if property.description}
				<div class="mt-5 border-t border-border pt-4">
					<h2 class="text-xs text-muted-foreground">{m.properties_description()}</h2>
					<p class="mt-1 whitespace-pre-wrap text-sm leading-6">{property.description}</p>
				</div>
			{/if}
		</section>

		{#if property.owners.length > 0}
			<section class="rounded-xl border border-border bg-card p-5">
				<h2 class="text-base font-semibold">{m.properties_owners()}</h2>
				<div class="mt-3 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
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
{:else}
	<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
		{m.properties_not_found()}
	</div>
{/if}

{#if property}
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
