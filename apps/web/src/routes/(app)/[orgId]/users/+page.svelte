<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { userQueries, userQueryKeys } from '#lib/features/users/queries';
	import AddUserSheet from '#lib/features/users/add-user-sheet.svelte';
	import UsersDirectory from '#lib/features/users/users-directory.svelte';
	import * as m from '#lib/paraglide/messages';
	import MessageBanner from '#lib/shared/components/message-banner.svelte';
	import type { User } from '#lib/features/auth/types';

	const app = getAppContext();
	const usersQuery = createQuery(() => ({
		queryKey: userQueryKeys.all(app.currentUser?.organization_id ?? ''),
		queryFn: userQueries.list,
		enabled: app.currentUser?.role === 'admin'
	}));
	const users = $derived(usersQuery.data ?? []);
</script>

<div class="flex items-start justify-between gap-4">
	<AddUserSheet busy={app.busy} onAdd={app.addUser} />
</div>
{#if usersQuery.error || app.error}
	<MessageBanner message={usersQuery.error ? String(usersQuery.error) : app.error} />
{/if}
<UsersDirectory
	{users}
	loading={usersQuery.isLoading}
	busy={app.busy}
	onToggle={(user: User) => void app.setUserActive(user)}
/>
