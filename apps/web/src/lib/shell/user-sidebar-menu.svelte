<script lang="ts">
	import { goto } from '$app/navigation';
	import { CircleUserRound, LogOut, Settings2 } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import DropdownMenu from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu.svelte';
	import DropdownMenuContent from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-content.svelte';
	import DropdownMenuItem from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-item.svelte';
	import DropdownMenuSeparator from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-separator.svelte';
	import DropdownMenuTrigger from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-trigger.svelte';
	import * as m from '#lib/paraglide/messages';
	import { getHtmlDirection, getInlineEndSide } from '#lib/core/i18n/direction';
	import type { User } from '#lib/features/auth/types';

	let {
		currentUser,
		onLogout
	}: {
		currentUser: User;
		onLogout: () => void;
	} = $props();

	const direction = $derived(getHtmlDirection());
	const menuSide = $derived(getInlineEndSide(direction));
	const initials = $derived(currentUser.name.trim().charAt(0).toLocaleUpperCase());
	const roleLabel = $derived(
		currentUser.role === 'admin'
			? m.users_role_admin()
			: currentUser.role === 'staff'
				? m.users_role_staff()
				: currentUser.role === 'readonly'
					? m.users_role_readonly()
				: currentUser.role
	);
</script>

<DropdownMenu dir={direction}>
	<DropdownMenuTrigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="ghost"
				size="icon"
				class="size-8 rounded-[10px] bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground"
				aria-label={m.common_user_menu()}
				title={currentUser.name}
			>
				<span class="text-xs font-bold">{initials}</span>
			</Button>
		{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuContent
		align="end"
		side={menuSide}
		sideOffset={12}
		dir={direction}
		class="w-64 p-1 text-start"
	>
		<div class="flex min-w-0 items-center gap-3 border-b border-border/50 px-3 py-2.5">
			<span class="grid size-9 shrink-0 place-items-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
				{initials}
			</span>
			<div class="flex min-w-0 flex-col gap-1">
				<span class="truncate text-xs font-semibold text-foreground">{currentUser.name}</span>
				<span class="truncate text-[10px] text-muted-foreground">{currentUser.email}</span>
				<span class="w-fit rounded bg-primary/10 px-1.5 py-0.5 text-[10px] font-medium text-primary">
					{roleLabel}
				</span>
			</div>
		</div>
		<div class="py-1">
			<DropdownMenuItem
				class="cursor-pointer rounded px-3 py-2 text-xs"
				onclick={() => void goto(`/${currentUser.organization_id}/account`)}
			>
				<CircleUserRound size={15} />
				{m.nav_account()}
			</DropdownMenuItem>
			<DropdownMenuItem
				class="cursor-pointer rounded px-3 py-2 text-xs"
				onclick={() => void goto(`/${currentUser.organization_id}/settings`)}
			>
				<Settings2 size={15} />
				{m.nav_settings()}
			</DropdownMenuItem>
			<DropdownMenuSeparator class="my-1" />
			<DropdownMenuItem
				variant="destructive"
				class="cursor-pointer rounded px-3 py-2 text-xs"
				onclick={() => void onLogout()}
			>
				<LogOut size={15} />
				{m.common_sign_out()}
			</DropdownMenuItem>
		</div>
	</DropdownMenuContent>
</DropdownMenu>
