<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import {
		Button,
		Sheet,
		SheetContent,
		SheetHeader,
		SheetTitle
	} from '@aqardesk/ui';
	import UserForm, { type UserFormValues } from '#lib/features/users/user-form.svelte';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		busy: boolean;
		roles: RolePermissionSet[];
		onAdd: (values: UserFormValues) => Promise<boolean>;
	};

	let { busy, roles, onAdd }: Props = $props();
	let open = $state(false);

	async function addUser(values: UserFormValues) {
		if (await onAdd(values)) open = false;
	}
</script>

<Button class="shrink-0 rounded-md" onclick={() => (open = true)}>
	<Plus size={16} />{m.users_add()}
</Button>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{m.users_add_title()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<UserForm
				{busy}
				{roles}
				onSubmit={async (values) => {
					await addUser(values);
					return !open;
				}}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
