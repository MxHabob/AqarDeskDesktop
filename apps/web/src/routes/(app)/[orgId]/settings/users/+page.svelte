<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { userQueries, userQueryKeys } from '#lib/features/users/queries';
	import AddUserSheet from '#lib/features/users/add-user-sheet.svelte';
	import UsersDirectory from '#lib/features/users/users-directory.svelte';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import { toast } from '@aqardesk/ui';

	const app = getAppContext();
	const usersQuery = createQuery(() => ({
		queryKey: userQueryKeys.all(app.currentUser?.organization_id ?? ''),
		queryFn: userQueries.list,
		enabled: app.currentUser?.role === 'admin'
	}));
	const users = $derived(usersQuery.data ?? []);
	let lastNotifiedError = '';
	$effect(() => {
		const error = usersQuery.error ? String(usersQuery.error) : '';
		if (error && error !== lastNotifiedError) toast.error(error);
		lastNotifiedError = error;
	});
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_user_management()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_user_management_description()}</p>
</header>
<div class="flex items-start justify-between gap-4">
	<AddUserSheet busy={app.busy} onAdd={app.addUser} />
</div>
<UsersDirectory
	{users}
	loading={usersQuery.isLoading}
	busy={app.busy}
	onToggle={(user: User) => void app.setUserActive(user)}
/>
