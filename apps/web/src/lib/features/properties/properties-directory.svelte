<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import type { Snippet } from 'svelte';
	import type { Property } from '@aqardesk/sdk';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		properties: Property[];
		loading: boolean;
		hasMore?: boolean;
		onloadmore?: () => Promise<unknown>;
		canCreate: boolean;
		canUpdate: boolean;
		canArchive: boolean;
		onAdd: () => void;
		onOpenDetails: (property: Property) => void;
		onEdit: (property: Property) => void;
		onArchive: (property: Property) => void;
	};
	let { properties, loading, hasMore = false, onloadmore, canCreate, canUpdate, canArchive, onAdd, onOpenDetails, onEdit, onArchive }: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'organization_id', header: m.settings_organization_id(), width: 220 },
		{ id: 'name', header: m.properties_name(), width: 180 },
		{ id: 'code', header: m.properties_code(), width: 150 },
		{
			id: 'property_type',
			header: m.properties_type(),
			type: 'enum',
			options: [
				{ value: 'residential', label: m.properties_type_residential() },
				{ value: 'commercial', label: m.properties_type_commercial() },
				{ value: 'mixed_use', label: m.properties_type_mixed() },
				{ value: 'land', label: m.properties_type_land() },
				{ value: 'other', label: m.common_other() },
				{ value: 'building', label: m.properties_type_building() },
				{ value: 'house_villa', label: m.properties_type_house_villa() },
				{ value: 'investment', label: m.properties_type_investment() },
				{ value: 'warehouse', label: m.properties_type_warehouse() },
				{ value: 'yard', label: m.properties_type_yard() },
				{ value: 'apartment', label: m.properties_type_apartment() }
			]
		},
		{ id: 'description', header: m.properties_description(), width: 220 },
		{ id: 'address_line_1', header: m.properties_address(), width: 200 },
		{ id: 'address_line_2', header: m.properties_address_line_2(), width: 200 },
		{ id: 'city', header: m.properties_city(), width: 140 },
		{ id: 'region', header: m.properties_region(), width: 140 },
		{ id: 'district', header: m.properties_district(), width: 140 },
		{ id: 'neighborhood', header: m.properties_neighborhood(), width: 150 },
		{ id: 'postal_code', header: m.properties_postal_code(), width: 120 },
		{ id: 'country_code', header: m.properties_country(), width: 120 },
		{ id: 'lot_area_square_meters', header: m.properties_lot_area(), type: 'decimal', width: 130 },
		{
			id: 'building_area_square_meters',
			header: m.properties_building_area(),
			type: 'decimal',
			width: 150
		},
		{ id: 'frontage_meters', header: m.properties_frontage(), type: 'decimal', width: 120 },
		{ id: 'depth_meters', header: m.properties_depth(), type: 'decimal', width: 120 },
		{ id: 'latitude', header: m.properties_latitude(), type: 'decimal', width: 120 },
		{ id: 'longitude', header: m.properties_longitude(), type: 'decimal', width: 120 },
		{
			id: 'owners',
			header: m.properties_owners(),
			width: 280,
			accessor: (row) => JSON.stringify(row.owners ?? [])
		},
		{
			id: 'status',
			header: m.properties_status(),
			type: 'status',
			options: [
				{ value: 'active', label: m.common_enabled() },
				{ value: 'inactive', label: m.common_disabled() },
				{ value: 'archived', label: m.common_disabled() }
			]
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{ id: 'updated_at', header: m.settings_updated_at(), type: 'datetime', width: 190 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'details', icon: 'Eye', label: m.properties_details(), variant: 'outline' },
					{ id: 'edit', icon: 'Pencil', label: m.properties_edit(), variant: 'outline', disabled: () => !canUpdate },
					{
						id: 'archive',
						icon: 'Trash2',
						label: m.properties_archive(),
						variant: 'destructive',
						disabled: (row: Record<string, unknown>) => !canArchive || row.status === 'archived'
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const property = properties.find((item) => item.id === String(event.row.id ?? ''));
		if (!property) return;
		if (event.actionId === 'dblclick') onOpenDetails(property);
		if (event.actionId === 'details') onOpenDetails(property);
		if (event.actionId === 'edit' && canUpdate) onEdit(property);
		if (event.actionId === 'archive' && canArchive) onArchive(property);
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={properties}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		{hasMore}
		{onloadmore}
		emptyTitle={m.properties_empty()}
		emptyDescription={m.dashboard_properties_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<Button class="rounded-md" disabled={!canCreate} onclick={onAdd}>
				<Plus size={16} />{m.properties_add()}
			</Button>
		{/snippet}
	</AqarDeskGrid>
</div>
