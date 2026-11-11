<script lang="ts">
	import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { Property, PropertyInput } from '@aqardesk/sdk';
	import PropertyForm from '#lib/features/properties/property-form.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		open?: boolean;
		initial?: Property;
		busy: boolean;
		onsave: (values: PropertyInput) => Promise<boolean>;
		onclose: () => void;
	};

	let { open = $bindable(false), initial, busy, onsave, onclose }: Props = $props();
</script>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{initial ? m.properties_edit() : m.properties_add()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<PropertyForm {initial} {busy} onSubmit={onsave} onCancel={onclose} />
		{/if}
	</SheetContent>
</Sheet>
