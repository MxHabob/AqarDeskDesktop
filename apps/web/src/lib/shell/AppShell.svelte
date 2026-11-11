<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Languages } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { AppLocale } from '#lib/core/i18n';
	import type { ViewKey } from '#lib/shell/navigation';
	import AppSidebar from '#lib/shell/app-sidebar.svelte';

	type Props = {
		active: ViewKey;
		title: string;
		currentUser: User;
		connected: boolean;
		locale: AppLocale;
		onSelect: (view: ViewKey) => void;
		onLogout: () => void;
		onToggleLanguage: () => void;
		children: Snippet;
	};

	let {
		active,
		title,
		currentUser,
		connected,
		locale,
		onSelect,
		onLogout,
		onToggleLanguage,
		children
	}: Props = $props();
</script>

<div class="flex h-dvh min-w-80 overflow-hidden bg-background font-sans text-foreground">
	<AppSidebar {active} {currentUser} {onSelect} {onLogout} />
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">
		<header class="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-border bg-card px-4 sm:px-6">
			<div class="flex min-w-0 items-center gap-2 text-sm">
				<span class="hidden text-muted-foreground sm:inline">{m.nav_workspace()}</span>
				<span class="hidden text-muted-foreground/50 sm:inline">/</span>
				<strong class="truncate font-medium text-foreground">{title}</strong>
			</div>
			<div class="flex shrink-0 items-center gap-3">
				<span class="hidden items-center gap-2 text-xs text-muted-foreground sm:inline-flex">
					<span class="size-2 rounded-full" class:bg-primary-foreground={connected} class:bg-muted-foreground={!connected}></span>
					{connected ? m.status_connected() : m.status_not_connected()}
				</span>
				<Button variant="outline" class="h-8 rounded-md" aria-label={m.common_language()} onclick={onToggleLanguage}>
					<Languages size={16} />
					<span>{locale === 'ar' ? m.language_english() : m.language_arabic()}</span>
				</Button>
			</div>
		</header>

		<nav class="flex shrink-0 gap-1 overflow-x-auto border-b border-border bg-card px-2 py-1 scrollbar-none md:hidden" aria-label={m.nav_workspace()}>
			<Button variant="ghost" class={active === 'overview' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('overview')}>{m.nav_overview()}</Button>
			<Button variant="ghost" class={active === 'properties' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('properties')}>{m.nav_properties()}</Button>
			<Button variant="ghost" class={active === 'units' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('units')}>{m.nav_units()}</Button>
			<Button variant="ghost" class={active === 'tenants' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('tenants')}>{m.nav_tenants()}</Button>
			<Button variant="ghost" class={active === 'contracts' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('contracts')}>{m.nav_contracts()}</Button>
			<Button variant="ghost" class={active === 'payments' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('payments')}>{m.nav_payments()}</Button>
			{#if currentUser.role === 'admin'}
				<Button variant="ghost" class={active === 'users' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('users')}>{m.nav_users()}</Button>
			{/if}
			<Button variant="ghost" class={active === 'settings' ? 'shrink-0 rounded-md bg-accent text-accent-foreground' : 'shrink-0 rounded-md text-muted-foreground'} onclick={() => onSelect('settings')}>{m.nav_settings()}</Button>
		</nav>

		<main class="min-h-0 flex-1 overflow-y-auto p-4 scrollbar-none sm:p-6 lg:p-8">
			<div class="mx-auto grid w-full max-w-6xl gap-5">
				{@render children()}
			</div>
		</main>
	</div>
</div>
