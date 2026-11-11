<script lang="ts">
	import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { Tenant, TenantInput } from '@aqardesk/sdk';
	import TenantForm from '#lib/features/tenants/tenant-form.svelte';
	import * as m from '#lib/paraglide/messages';
	type Props = {
		open?: boolean;
		initial?: Tenant;
		busy: boolean;
		onsave: (values: TenantInput) => Promise<boolean>;
		onclose: () => void;
	};
	let { open = $bindable(false), initial, busy, onsave, onclose }: Props = $props();
</script>
<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4"><SheetTitle>{initial ? m.tenants_edit() : m.tenants_add()}</SheetTitle></SheetHeader>
		{#if open}<TenantForm {initial} {busy} onSubmit={onsave} onCancel={onclose} />{/if}
	</SheetContent>
</Sheet>
