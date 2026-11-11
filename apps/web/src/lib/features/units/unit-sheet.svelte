<script lang="ts">
	import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { Property, PropertyUnit, PropertyUnitInput } from '@aqardesk/sdk';
	import UnitForm from '#lib/features/units/unit-form.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		open?: boolean;
		initial?: PropertyUnit;
		properties: Property[];
		busy: boolean;
		onsave: (values: PropertyUnitInput) => Promise<boolean>;
		onclose: () => void;
	};
	let { open = $bindable(false), initial, properties, busy, onsave, onclose }: Props = $props();
</script>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 border-b border-border px-4 py-4">
			<SheetTitle>{initial ? m.units_edit() : m.units_add()}</SheetTitle>
			<SheetDescription>{m.dashboard_properties_description()}</SheetDescription>
		</SheetHeader>
		{#if open}
			<UnitForm {initial} {properties} {busy} onSubmit={onsave} onCancel={onclose} />
		{/if}
	</SheetContent>
</Sheet>
