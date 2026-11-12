<script lang="ts">
	import { Button, Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import type { User } from '#lib/features/auth/types';
	import UserForm, { type UserFormValues } from '#lib/features/users/user-form.svelte';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let {
		user,
		busy,
		roles,
		onUpdate,
		open = $bindable(false)
	}: {
		user: User;
		busy: boolean;
		roles: RolePermissionSet[];
		onUpdate: (user: User, values: UserFormValues) => Promise<boolean>;
		open?: boolean;
	} = $props();
	const app = getAppContext();

	async function save(values: UserFormValues) {
		if (await onUpdate(user, values)) open = false;
	}
</script>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{m.users_edit_title()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<UserForm
				initial={user}
				{roles}
				roleLocked={app.currentUser?.id === user.id}
				{busy}
				onSubmit={async (values) => {
					await save(values);
					return !open;
				}}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
