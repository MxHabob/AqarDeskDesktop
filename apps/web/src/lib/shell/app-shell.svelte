<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import { AppWindow } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from '#lib/shell/navigation';
	import AppSidebar from '#lib/shell/app-sidebar.svelte';
	import SidebarNav from '#lib/shell/sidebar-nav.svelte';
	import MainViewport from '#lib/shell/main-viewport.svelte';

	type Props = {
		active: ViewKey;
		title: string;
		currentUser: User;
		organizationName: string;
		connected: boolean;
		onSelect: (view: ViewKey) => void;
		onLogout: () => void;
		children: Snippet;
	};

	let { active, title, currentUser, organizationName, connected, onSelect, onLogout, children }: Props = $props();
	let sidebarSize = $state(2);
	let desktopShell = $state(false);
	const showContextNavigation = $derived(
		active !== 'overview' && active !== 'account' && !page.params.propertyId
	);
	const settingsPath = $derived(`/${currentUser.organization_id}/settings`);
	const routePath = $derived(deLocalizeUrl(page.url).pathname);
	const gridWorkspace = $derived(
		[
			`/${currentUser.organization_id}/properties`,
			`/${currentUser.organization_id}/units`,
			`/${currentUser.organization_id}/tenants`,
			`/${currentUser.organization_id}/payments`,
			`/${currentUser.organization_id}/settings/users`,
			`/${currentUser.organization_id}/settings/currencies`,
			`/${currentUser.organization_id}/settings/audit`,
			`/${currentUser.organization_id}/settings/contract-templates`
		].includes(routePath)
	);
	const settingsSection = $derived(routePath.slice(settingsPath.length).replace(/^\/+/, ''));
	const settingsNavigation = $derived(
		active === 'settings'
			? {
					title: m.nav_settings(),
					activeKey:
						settingsSection || (currentUser.role === 'admin' ? 'organization' : 'general'),
					items: [
						{
							key: currentUser.role === 'admin' ? 'organization' : 'general',
							label:
								currentUser.role === 'admin'
									? m.settings_organization()
									: m.settings_general()
						},
						...(currentUser.role === 'admin'
							? [
									{ key: 'databases', label: m.nav_database_management() },
									{ key: 'users', label: m.settings_user_management() },
									{ key: 'audit', label: m.settings_audit() },
									{ key: 'currencies', label: m.settings_currencies() },
									{ key: 'contract-templates', label: m.settings_contract_templates() }
								]
							: [])
					]
				}
			: undefined
	);

	function navigateSettings(key: string) {
		void goto(
			key === 'organization' || key === 'general' ? settingsPath : `${settingsPath}/${key}`
		);
	}

	onMount(() => {
		desktopShell = '__TAURI_INTERNALS__' in window;
	});
</script>

<div class="flex h-full min-h-0 min-w-0 overflow-hidden bg-background font-sans text-foreground">
	<div class="hidden h-full md:flex">
		<AppSidebar {active} {currentUser} {organizationName} {onSelect} {onLogout} />
	</div>
	<MainViewport name="workspace-secondary-sidebar" bind:sidebarSize showLeft={showContextNavigation}>
		{#snippet left()}
			<SidebarNav
				{active}
				{currentUser}
				collapsed={sidebarSize === 0}
				{onSelect}
				onExpand={() => (sidebarSize = 2)}
				navigation={settingsNavigation}
				onNavigate={navigateSettings}
			/>
		{/snippet}
		{#snippet middle()}
			<div class="flex h-full min-h-0 min-w-0 flex-col overflow-hidden">
				{#if !desktopShell}
					<header class="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-border/70 bg-card px-4 sm:px-6">
						<div class="flex min-w-0 items-center gap-3">
							<span class="grid size-8 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
								<AppWindow size={16} />
							</span>
							<div class="min-w-0">
								<p class="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">{m.nav_workspace()}</p>
								<h1 class="truncate text-sm font-semibold text-foreground">{title}</h1>
							</div>
						</div>
						<div class="hidden items-center gap-2 text-xs text-muted-foreground sm:flex">
							<span class={`size-1.5 rounded-full ${connected ? 'bg-emerald-500' : 'bg-destructive'}`}></span>
							{connected ? m.status_connected() : m.status_not_connected()}
						</div>
					</header>
				{/if}
				<main
					class={gridWorkspace
						? 'flex min-h-0 flex-1 flex-col overflow-hidden p-3 scrollbar-none sm:p-4'
						: 'min-h-0 flex-1 overflow-y-auto p-4 scrollbar-none sm:p-6 lg:p-6'}
				>
					<div
						class={gridWorkspace
							? 'flex min-h-0 w-full flex-1 flex-col gap-3'
							: 'grid min-h-full w-full content-start gap-4'}
					>
						{@render children()}
					</div>
				</main>
			</div>
		{/snippet}
	</MainViewport>
</div>
