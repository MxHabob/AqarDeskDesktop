<script lang="ts">
	import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import RoleForm, { type RoleFormValues } from '#lib/features/roles/role-form.svelte';
	import * as m from '#lib/paraglide/messages';

	let {
		role,
		busy,
		onSubmit,
		open = $bindable(false)
	}: {
		role?: RolePermissionSet;
		busy: boolean;
		onSubmit: (values: RoleFormValues) => Promise<boolean>;
		open?: boolean;
	} = $props();
</script>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>
				{role ? m.settings_permissions_edit_role() : m.settings_permissions_add_role()}
			</SheetTitle>
		</SheetHeader>
		{#if open}
			<RoleForm
				initial={role}
				{busy}
				{onSubmit}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
