<script lang="ts">
	import {
		Building2,
		LayoutDashboard,
		ScrollText,
		Settings2
	} from '@lucide/svelte';
	import { Button, Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getHtmlDirection, getInlineEndSide } from '#lib/core/i18n/direction';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from '#lib/shell/navigation';
	import OrgSidebarSwitcher from '#lib/shell/org-sidebar-switcher.svelte';
	import UserSidebarMenu from '#lib/shell/user-sidebar-menu.svelte';
	import MulverseIntegrationPopover from '#lib/shell/mulverse-integration-popover.svelte';
	import MoreActionsDropdown from '#lib/shell/more-actions-dropdown.svelte';
	import SystemClock from '#lib/shell/system-clock.svelte';

	type Props = {
		active: ViewKey;
		currentUser: User;
		organizationName: string;
		onSelect: (view: ViewKey) => void;
		onLogout: () => void;
	};

	let { active, currentUser, organizationName, onSelect, onLogout }: Props = $props();
	const tooltipSide = $derived(getInlineEndSide(getHtmlDirection()));

	const links = [
		{ key: 'overview', label: m.nav_overview(), icon: LayoutDashboard, active: ['overview'] },
		{
			key: 'properties',
			label: m.nav_properties(),
			icon: Building2,
			active: ['properties', 'units', 'tenants']
		},
		{
			key: 'contracts',
			label: m.nav_contracts(),
			icon: ScrollText,
			active: ['contracts', 'payments']
		},
		{
			key: 'settings',
			label: m.nav_settings(),
			icon: Settings2,
			active: ['users', 'settings', 'account']
		}
	] satisfies { key: ViewKey; label: string; icon: typeof Building2; active: string[] }[];
</script>

<TooltipProvider delayDuration={150}>
	<aside class="flex h-full w-14 shrink-0 flex-col items-center justify-between py-3 select-none">
		<div class="flex flex-col items-center gap-3">
			<OrgSidebarSwitcher
				organizationId={currentUser.organization_id}
				{organizationName}
				canManageBilling={currentUser.role === 'admin'}
			/>

			<div class="h-px w-8 bg-border/80"></div>

			<nav class="flex flex-col items-center gap-1" aria-label={m.nav_workspace()}>
				{#each links as item (item.key)}
					{@const Icon = item.icon}
					<Tooltip>
						<TooltipTrigger>
							<Button
								variant={item.active.includes(active) ? 'default' : 'ghost'}
								size="icon"
								class={`relative size-9 rounded-md after:absolute after:inset-y-2 after:start-0 after:w-0.5 after:rounded-full after:bg-primary after:content-[''] ${
									item.active.includes(active) ? 'after:opacity-100' : 'after:opacity-0'
								}`}
								aria-label={item.label}
								aria-current={item.active.includes(active) ? 'page' : undefined}
								onclick={() => onSelect(item.key)}
							>
								<Icon size={19} />
							</Button>
						</TooltipTrigger>
						<TooltipContent side={tooltipSide} sideOffset={10}>{item.label}</TooltipContent>
					</Tooltip>
				{/each}
			</nav>
		</div>

		<div class="flex flex-col items-center gap-2">
			<div class="flex flex-col items-center gap-1">
				<MulverseIntegrationPopover orgId={currentUser.organization_id} />
				<MoreActionsDropdown
					orgId={currentUser.organization_id}
					isAdmin={currentUser.role === 'admin'}
				/>
				<SystemClock />
			</div>
			<UserSidebarMenu {currentUser} onLogout={onLogout} />
		</div>
	</aside>
</TooltipProvider>
