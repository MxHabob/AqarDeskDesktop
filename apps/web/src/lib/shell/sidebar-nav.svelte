<script lang="ts">
	import { Button } from '@aqardesk/ui';
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from '#lib/shell/navigation';

	type Props = {
		active: ViewKey;
		currentUser: User;
		collapsed: boolean;
		onSelect: (view: ViewKey) => void;
		onExpand: () => void;
	};

	let { active, currentUser, collapsed, onSelect, onExpand }: Props = $props();

	const group = $derived.by(() => {
		if (active === 'properties' || active === 'units' || active === 'tenants') {
			return {
				title: m.nav_properties(),
				items: [
					{ key: 'properties' as const, label: m.nav_properties() },
					{ key: 'units' as const, label: m.nav_units() },
					{ key: 'tenants' as const, label: m.nav_tenants() }
				]
			};
		}
		if (active === 'contracts' || active === 'payments') {
			return {
				title: m.nav_contracts(),
				items: [
					{ key: 'contracts' as const, label: m.nav_contracts() },
					{ key: 'payments' as const, label: m.nav_payments() }
				]
			};
		}
		if (active === 'users' || active === 'settings') {
			return {
				title: m.nav_settings(),
				items: [
					...(currentUser.role === 'admin'
						? [{ key: 'users' as const, label: m.nav_users() }]
						: []),
					{ key: 'settings' as const, label: m.nav_settings() }
				]
			};
		}
		return {
			title: m.nav_workspace(),
			items: [
				{ key: 'properties' as const, label: m.nav_properties() },
				{ key: 'tenants' as const, label: m.nav_tenants() },
				{ key: 'contracts' as const, label: m.nav_contracts() },
				{ key: 'payments' as const, label: m.nav_payments() }
			]
		};
	});
</script>

<aside class="flex h-full min-h-0 flex-col overflow-hidden bg-card/60">
	{#if collapsed}
		<div class="flex flex-1 flex-col items-center gap-3 py-3">
			<Button
				variant="ghost"
				size="icon"
				class="size-9 rounded-lg"
				aria-label={m.sidebar_expand()}
				title={m.sidebar_expand()}
				onclick={onExpand}
			>
				<ChevronLeft size={17} class="rtl:hidden" />
				<ChevronRight size={17} class="hidden rtl:block" />
			</Button>
		</div>
	{:else}
		<div class="flex h-14 shrink-0 items-center gap-3 border-b border-border/60 px-4">
			<span class="grid size-8 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
				<span class="text-xs font-bold">{currentUser.name.slice(0, 1).toLocaleUpperCase()}</span>
			</span>
			<div class="min-w-0 flex-1">
				<p class="truncate text-sm font-semibold text-foreground">{m.app_name()}</p>
				<p class="truncate text-[11px] text-muted-foreground">{currentUser.name}</p>
			</div>
		</div>

		<nav class="min-h-0 flex-1 overflow-y-auto px-3 py-4 scrollbar-none" aria-label={m.nav_workspace()}>
			<section>
				<h2 class="mb-1.5 px-3 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
					{group.title}
				</h2>
				<div class="grid gap-1">
					{#each group.items as item (item.key)}
						<Button
							variant="ghost"
							class={active === item.key
								? 'h-9 justify-start rounded-lg bg-accent px-3 text-start text-accent-foreground'
								: 'h-9 justify-start rounded-lg px-3 text-start text-muted-foreground hover:bg-muted hover:text-foreground'}
							aria-current={active === item.key ? 'page' : undefined}
							onclick={() => onSelect(item.key)}
						>
							<span class="truncate">{item.label}</span>
						</Button>
					{/each}
				</div>
			</section>
		</nav>
	{/if}
</aside>
