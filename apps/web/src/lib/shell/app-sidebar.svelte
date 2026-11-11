<script lang="ts">
	import {
		Building2,
		LayoutDashboard,
		LogOut,
		WalletCards,
		Settings2,
	} from '@lucide/svelte';
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

	const links = [
		{ key: 'overview', label: m.nav_overview(), icon: LayoutDashboard, active: ['overview'] },
		{ key: 'properties', label: m.nav_properties(), icon: Building2, active: ['properties', 'units', 'tenants'] },
		{ key: 'contracts', label: m.nav_contracts(), icon: WalletCards, active: ['contracts', 'payments'] },
		{ key: 'settings', label: m.nav_settings(), icon: Settings2, active: ['users', 'settings'] }
	] satisfies { key: ViewKey; label: string; icon: typeof Building2; active: string[] }[];
</script>

<aside class="flex h-full w-14 shrink-0 flex-col items-center border-e border-border/70 bg-card py-3">
	<a
		class="mb-5 grid size-9 place-items-center rounded-xl bg-primary text-primary-foreground shadow-card"
		href={`/${currentUser.organization_id}`}
		aria-label={m.app_name()}
		title={m.app_name()}
	>
		<Building2 size={19} />
	</a>

	<nav class="flex w-full flex-1 flex-col items-center gap-1.5" aria-label={m.nav_workspace()}>
		{#each links as item (item.key)}
			{@const Icon = item.icon}
			<Button
				variant="ghost"
				size="icon"
				class={item.active.includes(active)
					? 'relative size-10 rounded-xl bg-accent text-accent-foreground after:absolute after:inset-y-2 after:start-0 after:w-0.5 after:rounded-full after:bg-primary'
					: 'size-10 rounded-xl text-muted-foreground hover:bg-muted hover:text-foreground'}
				aria-label={item.label}
				aria-current={item.active.includes(active) ? 'page' : undefined}
				title={item.label}
				onclick={() => onSelect(item.key)}
			>
				<Icon size={18} />
			</Button>
		{/each}
	</nav>

	<div class="mt-3 flex w-full flex-col items-center gap-2 border-t border-border/70 pt-3">
		<span
			class="grid size-9 place-items-center rounded-full bg-secondary text-sm font-semibold text-secondary-foreground"
			title={currentUser.name}
		>
			{currentUser.name.slice(0, 1).toLocaleUpperCase()}
		</span>
		<Button
			variant="ghost"
			size="icon"
			class="size-9 rounded-xl text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
			aria-label={m.common_sign_out()}
			title={m.common_sign_out()}
			onclick={onLogout}
		>
			<LogOut size={17} />
		</Button>
	</div>
</aside>
