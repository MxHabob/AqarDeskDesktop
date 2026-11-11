<script lang="ts">
	import { UsersRound } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { User } from '#lib/features/auth/types';
	import * as m from '#lib/paraglide/messages';
	import EmptyState from '#lib/shared/components/empty-state.svelte';
	import StatusBadge from '#lib/shared/components/status-badge.svelte';

	type Props = {
		users: User[];
		loading: boolean;
		busy: boolean;
		onToggle: (user: User) => void;
	};

	let { users, loading, busy, onToggle }: Props = $props();
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
	{#if loading}
		<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
	{:else if users.length === 0}
		<EmptyState title={m.users_empty()} description={m.users_description()} />
	{:else}
		<div class="overflow-x-auto scrollbar-none">
			<table class="w-full min-w-[35rem] border-collapse text-start text-sm">
				<thead class="border-b border-border text-xs font-medium text-muted-foreground">
					<tr>
						<th class="px-2 py-3 text-start">{m.users_name()}</th>
						<th class="px-2 py-3 text-start">{m.users_role()}</th>
						<th class="px-2 py-3 text-start">{m.users_status()}</th>
						<th class="px-2 py-3 text-start">{m.users_action()}</th>
					</tr>
				</thead>
				<tbody>
					{#each users as user (user.id)}
						<tr class="border-b border-border/60 last:border-0">
							<td class="px-2 py-3">
								<div class="grid gap-1">
									<strong class="font-medium text-foreground">{user.name}</strong>
									<span class="text-xs text-muted-foreground" dir="ltr">{user.email}</span>
								</div>
							</td>
							<td class="px-2 py-3 text-muted-foreground">
								{user.role === 'admin' ? m.users_role_admin() : m.users_role_staff()}
							</td>
							<td class="px-2 py-3">
								<StatusBadge
									label={user.is_active ? m.common_enabled() : m.common_disabled()}
									active={user.is_active}
								/>
							</td>
							<td class="px-2 py-3">
								<Button
									variant="outline"
									class="h-8 rounded-md"
									disabled={busy}
									onclick={() => onToggle(user)}
								>
									{user.is_active ? m.users_deactivate() : m.users_activate()}
								</Button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</Card>
