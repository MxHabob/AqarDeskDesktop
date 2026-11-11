<script lang="ts">
	import { Home, Pencil, Trash2 } from '@lucide/svelte';
	import type { Property, PropertyUnit, UnitType } from '@aqardesk/sdk';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import EmptyState from '#lib/shared/components/empty-state.svelte';
	import StatusBadge from '#lib/shared/components/status-badge.svelte';
	import * as m from '#lib/paraglide/messages';

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
	function unitTypeLabel(type: UnitType) {
		switch (type) {
			case 'apartment': return m.units_type_apartment();
			case 'office': return m.units_type_office();
			case 'retail': return m.units_type_retail();
			case 'warehouse': return m.units_type_warehouse();
			case 'parking': return m.units_type_parking();
			case 'other': return m.common_other();
		}
	}
</script>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1"><CardTitle class="text-base">{m.nav_units()}</CardTitle><CardDescription>{units.length} {m.nav_units()}</CardDescription></div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><Home size={18} /></span>
	</CardHeader>
	{#if loading}<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
	{:else if units.length === 0}<EmptyState title={m.units_empty()} description={m.dashboard_properties_description()} />
	{:else}
		<div class="overflow-x-auto scrollbar-none"><table class="w-full min-w-[38rem] border-collapse text-start text-sm">
			<thead class="border-b border-border text-xs font-medium text-muted-foreground"><tr><th class="px-2 py-3 text-start">{m.units_number()}</th><th class="px-2 py-3 text-start">{m.units_property()}</th><th class="px-2 py-3 text-start">{m.units_type()}</th><th class="px-2 py-3 text-start">{m.units_status()}</th><th class="px-2 py-3 text-start">{m.users_action()}</th></tr></thead>
			<tbody>{#each units as unit (unit.id)}
				<tr class="border-b border-border/60 last:border-0">
					<td class="px-2 py-3 font-medium">{unit.unit_number}</td><td class="px-2 py-3 text-muted-foreground">{propertyNames.get(unit.property_id) ?? ''}</td><td class="px-2 py-3 text-muted-foreground">{unitTypeLabel(unit.unit_type)}</td>
					<td class="px-2 py-3"><StatusBadge label={unit.status === 'available' ? m.units_status_available() : unit.status === 'maintenance' ? m.units_status_maintenance() : m.common_disabled()} active={unit.status === 'available'} /></td>
					<td class="px-2 py-3"><div class="flex gap-2"><Button variant="outline" size="icon" class="size-8" disabled={readonly} aria-label={m.units_edit()} onclick={() => onEdit(unit)}><Pencil size={15} /></Button><Button variant="outline" size="icon" class="size-8 text-destructive" disabled={readonly} aria-label={m.units_archive()} onclick={() => onArchive(unit)}><Trash2 size={15} /></Button></div></td>
				</tr>
			{/each}</tbody>
		</table></div>
	{/if}
</Card>
