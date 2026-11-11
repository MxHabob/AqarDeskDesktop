<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { Button, Card } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
	const app = getAppContext();
	const basePath = $derived(`/${data.orgId}/settings`);
	const routePath = $derived(deLocalizeUrl(page.url).pathname);
	const section = $derived(routePath.slice(basePath.length).replace(/^\/+/, ''));
	const adminOnly = ['users', 'audit', 'contract-templates'];
	const groups = $derived([
		{
			title: m.settings_group_general(),
			items: [
				{ path: '', label: m.settings_overview() },
				{ path: 'general', label: m.settings_general() }
			]
		},
		{
			title: m.settings_group_organization(),
			items: [{ path: 'organization', label: m.settings_organization() }]
		},
		{
			title: m.settings_group_account(),
			items: [
				{ path: 'profile', label: m.settings_my_profile() },
				...(app.currentUser?.role === 'admin'
					? [{ path: 'users', label: m.settings_user_management() }]
					: [])
			]
		},
		...(app.currentUser?.role === 'admin'
			? [
					{
						title: m.settings_group_operations(),
						items: [
							{ path: 'audit', label: m.settings_audit() },
							{ path: 'contract-templates', label: m.settings_contract_templates() }
						]
					}
				]
			: [])
	]);

	$effect(() => {
		if (!app.booting && app.currentUser && adminOnly.includes(section) && app.currentUser.role !== 'admin') {
			void goto(`${basePath}/general`, { replaceState: true });
		}
	});

	function navigate(path: string) {
		void goto(path ? `${basePath}/${path}` : basePath);
	}
</script>

<div class="grid min-h-full min-w-0 content-start gap-4 md:grid-cols-[13rem_minmax(0,1fr)]">
	<Card class="h-fit min-w-0 rounded-lg bg-card p-3 shadow-card ring-border/50">
		<p class="mb-3 px-2 text-sm font-semibold">{m.nav_settings()}</p>
		<nav class="grid gap-4" aria-label={m.nav_settings()}>
			{#each groups as group (group.title)}
				<section class="grid gap-1">
					<h2 class="px-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
						{group.title}
					</h2>
					{#each group.items as item (item.path)}
						<Button
							variant="ghost"
							class={section === item.path
								? 'h-9 justify-start rounded-md bg-accent px-2 text-start text-accent-foreground'
								: 'h-9 justify-start rounded-md px-2 text-start text-muted-foreground'}
							aria-current={section === item.path ? 'page' : undefined}
							onclick={() => navigate(item.path)}
						>
							<span class="truncate">{item.label}</span>
						</Button>
					{/each}
				</section>
			{/each}
		</nav>
	</Card>
	<div class="grid min-w-0 content-start gap-4">
		{@render children()}
	</div>
</div>
