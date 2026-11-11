<script lang="ts">
	import { Home } from '@lucide/svelte';
	import type { Property, PropertyUnit } from '@aqardesk/sdk';
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		units: PropertyUnit[];
		properties: Property[];
		loading: boolean;
		readonly: boolean;
		onEdit: (unit: PropertyUnit) => void;
		onArchive: (unit: PropertyUnit) => void;
	};
	let { units, properties, loading, readonly, onEdit, onArchive }: Props = $props();
	const propertyNames = $derived(new Map(properties.map((property) => [property.id, property.name])));
	const rtl = $derived(getLocale() === 'ar');
	const columns: OrbisColumnDef[] = $derived([
		{ id: 'unit_number', header: m.units_number(), width: 120 },
		{
			id: 'property_name',
			header: m.units_property(),
			accessor: (row) => propertyNames.get(String(row.property_id)) ?? ''
		},
		{
			id: 'unit_type',
			header: m.units_type(),
			type: 'enum',
			options: [
				{ value: 'apartment', label: m.units_type_apartment() },
				{ value: 'office', label: m.units_type_office() },
				{ value: 'retail', label: m.units_type_retail() },
				{ value: 'warehouse', label: m.units_type_warehouse() },
				{ value: 'parking', label: m.units_type_parking() },
				{ value: 'other', label: m.common_other() }
			]
		},
		{
			id: 'status',
			header: m.units_status(),
			type: 'status',
			options: [
				{ value: 'available', label: m.units_status_available() },
				{ value: 'maintenance', label: m.units_status_maintenance() },
				{ value: 'inactive', label: m.common_disabled() }
			]
		},
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'edit', icon: 'Pencil', label: m.units_edit(), variant: 'outline', disabled: () => readonly },
					{ id: 'archive', icon: 'Trash2', label: m.units_archive(), variant: 'destructive', disabled: () => readonly }
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const unit = units.find((item) => item.id === String(event.row.id ?? ''));
		if (!unit) return;
		if (event.actionId === 'edit') onEdit(unit);
		if (event.actionId === 'archive') onArchive(unit);
	}
</script>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1"><CardTitle class="text-base">{m.nav_units()}</CardTitle><CardDescription>{units.length} {m.nav_units()}</CardDescription></div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><Home size={18} /></span>
	</CardHeader>
	<OrbisGrid
		{columns}
		source={units}
		{loading}
		{rtl}
		readonly
		height="360px"
		emptyTitle={m.units_empty()}
		emptyDescription={m.dashboard_properties_description()}
		onaction={handleGridAction}
	/>
</Card>
