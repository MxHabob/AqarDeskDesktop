<script lang="ts">
	import { Building2, FileText, LayoutDashboard, LogOut, Settings2, UsersRound, WalletCards } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from '#lib/shell/navigation';

	type Props = {
		active: ViewKey;
		currentUser: User;
		onSelect: (view: ViewKey) => void;
		onLogout: () => void;
	};

	let { active, currentUser, onSelect, onLogout }: Props = $props();

	const links: { key: ViewKey; label: string; icon: typeof Building2 }[] = [
		{ key: 'overview', label: m.nav_overview(), icon: LayoutDashboard },
		{ key: 'properties', label: m.nav_properties(), icon: Building2 },
		{ key: 'units', label: m.nav_units(), icon: Building2 },
		{ key: 'tenants', label: m.nav_tenants(), icon: UsersRound },
		{ key: 'contracts', label: m.nav_contracts(), icon: FileText },
		{ key: 'payments', label: m.nav_payments(), icon: WalletCards },
		{ key: 'users', label: m.nav_users(), icon: UsersRound },
		{ key: 'settings', label: m.nav_settings(), icon: Settings2 }
	];
</script>

<aside class="hidden h-full w-64 shrink-0 flex-col border-e border-slate-200 bg-white p-4 md:flex">
	<a class="mb-8 flex items-center gap-3 px-2 py-1 text-slate-900 no-underline" href={`/${currentUser.organization_id}`} aria-label={m.app_name()}>
		<span class="grid size-10 place-items-center rounded-lg bg-emerald-800 text-white"><Building2 size={20} /></span>
		<span class="grid gap-0.5">
			<strong class="text-sm font-semibold">{m.app_name()}</strong>
			<small class="text-xs text-slate-500">{m.app_tagline()}</small>
		</span>
	</a>

	<p class="mb-2 px-3 text-xs font-semibold tracking-wide text-slate-500">{m.nav_workspace()}</p>
	<nav class="grid gap-1" aria-label={m.nav_workspace()}>
		{#each links as item (item.key)}
			{#if item.key !== 'users' || currentUser.role === 'admin'}
				{@const Icon = item.icon}
				<Button
					variant="ghost"
					class={active === item.key
						? 'h-10 justify-start rounded-md bg-emerald-50 px-3 text-emerald-900 hover:bg-emerald-100'
						: 'h-10 justify-start rounded-md px-3 text-slate-600 hover:bg-slate-100'}
					aria-current={active === item.key ? 'page' : undefined}
					onclick={() => onSelect(item.key)}
				>
					<Icon size={17} />{item.label}
				</Button>
			{/if}
		{/each}
	</nav>

	<div class="mt-auto grid gap-3 border-t border-slate-200 pt-4">
		<div class="flex min-w-0 items-center gap-3 px-1">
			<span class="grid size-9 shrink-0 place-items-center rounded-full bg-emerald-100 text-sm font-semibold text-emerald-900">
				{currentUser.name.slice(0, 1).toLocaleUpperCase()}
			</span>
			<span class="grid min-w-0">
				<strong class="truncate text-sm font-medium text-slate-800">{currentUser.name}</strong>
				<small class="truncate text-xs text-slate-500">{currentUser.email}</small>
			</span>
		</div>
		<Button variant="ghost" class="h-9 justify-start rounded-md px-3 text-slate-600" onclick={onLogout}>
			<LogOut size={16} />{m.common_sign_out()}
		</Button>
	</div>
</aside>
