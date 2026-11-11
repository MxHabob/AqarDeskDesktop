<script lang="ts">
	import { UserRound } from '@lucide/svelte';
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';
	import SettingsCapabilityNotice from '#lib/features/settings/settings-capability-notice.svelte';

	const app = getAppContext();
	const roleLabel = $derived(
		app.currentUser?.role === 'admin'
			? m.users_role_admin()
			: app.currentUser?.role === 'staff'
				? m.users_role_staff()
				: m.users_role_readonly()
	);
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_my_profile()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_my_profile_description()}</p>
</header>
{#if app.currentUser}
	<Card class="rounded-lg bg-card p-5 shadow-card ring-border/50">
		<CardHeader class="mb-4 flex-row items-center gap-3 p-0">
			<span class="grid size-10 place-items-center rounded-full bg-primary text-primary-foreground">
				<UserRound size={18} />
			</span>
			<div class="grid gap-1">
				<CardTitle class="text-base">{app.currentUser.name}</CardTitle>
				<CardDescription dir="ltr">{app.currentUser.email}</CardDescription>
			</div>
		</CardHeader>
		<div class="grid gap-2 text-sm sm:grid-cols-2">
			<div class="rounded-md border border-border p-3">
				<p class="text-xs text-muted-foreground">{m.users_role()}</p>
				<p class="mt-1 font-medium">{roleLabel}</p>
			</div>
			<div class="rounded-md border border-border p-3">
				<p class="text-xs text-muted-foreground">{m.org_sidebar_current()}</p>
				<p class="mt-1 font-medium" dir="ltr">{app.currentUser.organization_id}</p>
			</div>
		</div>
	</Card>
	<SettingsCapabilityNotice
		title={m.settings_profile_editing()}
		description={m.settings_profile_editing_unavailable()}
	/>
{/if}
