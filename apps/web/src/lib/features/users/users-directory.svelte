<script lang="ts">
	import { UsersRound } from '@lucide/svelte';
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { User } from '#lib/features/auth/types';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		users: User[];
		loading: boolean;
		busy: boolean;
		onToggle: (user: User) => void;
	};

	let { users, loading, busy, onToggle }: Props = $props();
	const rtl = $derived(getLocale() === 'ar');
	const columns: OrbisColumnDef[] = $derived([
		{ id: 'name', header: m.users_name(), width: 180 },
		{ id: 'email', header: m.auth_email(), width: 220 },
		{
			id: 'role',
			header: m.users_role(),
			type: 'enum',
			options: [
				{ value: 'admin', label: m.users_role_admin() },
				{ value: 'staff', label: m.users_role_staff() }
			]
		},
		{
			id: 'status',
			header: m.users_status(),
			type: 'status',
			accessor: (row) => row.is_active ? 'active' : 'inactive',
			options: [
				{ value: 'active', label: m.common_enabled() },
				{ value: 'inactive', label: m.common_disabled() }
			]
		},
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'deactivate',
						icon: 'Shield',
						label: m.users_deactivate(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active === true,
						disabled: () => busy
					},
					{
						id: 'activate',
						icon: 'Check',
						label: m.users_activate(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active !== true,
						disabled: () => busy
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		if (event.actionId !== 'activate' && event.actionId !== 'deactivate') return;
		const user = users.find((item) => item.id === String(event.row.id ?? ''));
		if (user) onToggle(user);
	}
</script>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1">
			<CardTitle class="text-base">{m.nav_users()}</CardTitle>
			<CardDescription>{users.length} {m.nav_users()}</CardDescription>
		</div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground">
			<UsersRound size={18} />
		</span>
	</CardHeader>
	<OrbisGrid
		{columns}
		source={users}
		{loading}
		{rtl}
		readonly
		height="360px"
		emptyTitle={m.users_empty()}
		emptyDescription={m.users_description()}
		onaction={handleGridAction}
	/>
</Card>
