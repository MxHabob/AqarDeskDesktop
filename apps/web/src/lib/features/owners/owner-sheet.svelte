<script lang="ts">
	import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { Owner, OwnerInput } from '@aqardesk/sdk';
	import OwnerForm from '#lib/features/owners/owner-form.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		open?: boolean;
		initial?: Owner;
		busy: boolean;
		onsave: (values: OwnerInput) => Promise<boolean>;
		onclose: () => void;
	};

	let { open = $bindable(false), initial, busy, onsave, onclose }: Props = $props();
</script>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{initial ? m.owners_edit() : m.owners_add()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<OwnerForm {initial} {busy} onSubmit={onsave} onCancel={onclose} />
		{/if}
	</SheetContent>
</Sheet>
