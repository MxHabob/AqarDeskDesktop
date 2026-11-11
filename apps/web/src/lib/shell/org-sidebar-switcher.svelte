<script lang="ts">
	import { goto } from '$app/navigation';
	import { Settings2 } from '@lucide/svelte';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';
	import { Button } from '@aqardesk/ui';
	import DropdownMenu from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu.svelte';
	import DropdownMenuContent from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-content.svelte';
	import DropdownMenuItem from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-item.svelte';
	import DropdownMenuSeparator from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-separator.svelte';
	import DropdownMenuTrigger from '@aqardesk/ui/components/ui/dropdown-menu/dropdown-menu-trigger.svelte';
	import * as m from '#lib/paraglide/messages';

	let { organizationId, organizationName }: { organizationId: string; organizationName: string } = $props();
	const organizationLabel = $derived(organizationName.trim() || m.app_name());
</script>

<DropdownMenu>
	<DropdownMenuTrigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="ghost"
				size="icon"
				class="size-8 rounded-[10px] bg-background text-foreground hover:bg-muted"
				aria-label={organizationLabel}
				title={organizationLabel}
			>
				<AqarDeskLogo size={26} class="text-foreground" />
			</Button>
		{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuContent align="start" sideOffset={12} class="w-56 p-1">
		<div class="flex items-center gap-3 border-b border-border/50 px-3 py-2.5">
			<span class="grid size-8 shrink-0 place-items-center rounded-lg bg-primary/10 text-primary">
				<AqarDeskLogo size={24} class="text-primary" />
			</span>
			<div class="flex min-w-0 flex-col gap-1">
				<span class="truncate text-xs font-semibold text-foreground" title={organizationLabel}>
					{organizationLabel}
				</span>
				<span class="text-[10px] text-muted-foreground">{m.org_sidebar_current()}</span>
			</div>
		</div>
		<div class="py-1">
			<DropdownMenuItem
				class="cursor-pointer rounded px-3 py-2 text-xs"
				onclick={() => void goto(`/${organizationId}`, { replaceState: true })}
			>
				<AqarDeskLogo size={18} class="text-muted-foreground" />
				{m.nav_overview()}
			</DropdownMenuItem>
			<DropdownMenuSeparator class="my-1" />
			<DropdownMenuItem
				class="cursor-pointer rounded px-3 py-2 text-xs"
				onclick={() => void goto(`/${organizationId}/settings`)}
			>
				<Settings2 size={15} />
				{m.nav_settings()}
			</DropdownMenuItem>
		</div>
	</DropdownMenuContent>
</DropdownMenu>
