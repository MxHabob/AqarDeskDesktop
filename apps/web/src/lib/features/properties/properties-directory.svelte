<script lang="ts">
	import { Building2, Pencil, Trash2 } from '@lucide/svelte';
	import type { Property, PropertyType } from '@aqardesk/sdk';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import EmptyState from '#lib/shared/components/empty-state.svelte';
	import StatusBadge from '#lib/shared/components/status-badge.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		properties: Property[];
		loading: boolean;
		readonly: boolean;
		onEdit: (property: Property) => void;
		onArchive: (property: Property) => void;
	};
	let { properties, loading, readonly, onEdit, onArchive }: Props = $props();

	function propertyTypeLabel(type: PropertyType) {
		switch (type) {
			case 'residential': return m.properties_type_residential();
			case 'commercial': return m.properties_type_commercial();
			case 'mixed_use': return m.properties_type_mixed();
			case 'land': return m.properties_type_land();
			case 'other': return m.common_other();
		}
	}
</script>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1">
			<CardTitle class="text-base">{m.nav_properties()}</CardTitle>
			<CardDescription>{properties.length} {m.nav_properties()}</CardDescription>
		</div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><Building2 size={18} /></span>
	</CardHeader>
	{#if loading}
		<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
	{:else if properties.length === 0}
		<EmptyState title={m.properties_empty()} description={m.dashboard_properties_description()} />
	{:else}
		<div class="overflow-x-auto scrollbar-none">
			<table class="w-full min-w-[42rem] border-collapse text-start text-sm">
				<thead class="border-b border-border text-xs font-medium text-muted-foreground"><tr>
					<th class="px-2 py-3 text-start">{m.properties_name()}</th><th class="px-2 py-3 text-start">{m.properties_type()}</th><th class="px-2 py-3 text-start">{m.properties_city()}</th><th class="px-2 py-3 text-start">{m.properties_status()}</th><th class="px-2 py-3 text-start">{m.users_action()}</th>
				</tr></thead>
				<tbody>
					{#each properties as property (property.id)}
						<tr class="border-b border-border/60 last:border-0">
							<td class="px-2 py-3"><strong class="font-medium">{property.name}</strong><div class="text-xs text-muted-foreground">{property.code} · {property.address_line_1}</div></td>
							<td class="px-2 py-3 text-muted-foreground">{propertyTypeLabel(property.property_type)}</td>
							<td class="px-2 py-3 text-muted-foreground">{property.city}</td>
							<td class="px-2 py-3"><StatusBadge label={property.status === 'active' ? m.common_enabled() : m.common_disabled()} active={property.status === 'active'} /></td>
							<td class="px-2 py-3"><div class="flex gap-2">
								<Button variant="outline" size="icon" class="size-8" disabled={readonly} aria-label={m.properties_edit()} onclick={() => onEdit(property)}><Pencil size={15} /></Button>
								<Button variant="outline" size="icon" class="size-8 text-destructive" disabled={readonly || property.status === 'archived'} aria-label={m.properties_archive()} onclick={() => onArchive(property)}><Trash2 size={15} /></Button>
							</div></td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</Card>
