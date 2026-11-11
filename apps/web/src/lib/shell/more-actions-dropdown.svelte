<script lang="ts">
	import {
		CircleHelp,
		Code2,
		FileText,
		Globe2,
		Headphones,
		LockKeyhole,
		MessageCircleMore,
		MoreHorizontal,
		Scale,
		UsersRound
	} from '@lucide/svelte';
	import {
		Button,
		Tooltip,
		TooltipContent,
		TooltipTrigger,
		toast
	} from '@aqardesk/ui';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuPortal,
		DropdownMenuSeparator,
		DropdownMenuSub,
		DropdownMenuSubContent,
		DropdownMenuSubTrigger,
		DropdownMenuTrigger
	} from '@aqardesk/ui/components/ui/dropdown-menu';
	import * as m from '#lib/paraglide/messages';
	import { openExternalUrl } from '#lib/core/external-url';
	import { getHtmlDirection, getInlineEndSide } from '#lib/core/i18n/direction';
	import { AboutDialog, BackupCliDialog, SupportDialog } from '#lib/features/system';

	let {
		orgId,
		isAdmin
	}: {
		orgId: string;
		isAdmin: boolean;
	} = $props();

	let isBackupCliOpen = $state(false);
	let isAboutOpen = $state(false);
	let isSupportOpen = $state(false);
	const direction = $derived(getHtmlDirection());
	const tooltipSide = $derived(getInlineEndSide(direction));
	const menuItemClass = 'flex cursor-pointer items-center gap-2.5 px-2.5 py-2 text-xs';
	const subTriggerClass =
		'flex cursor-pointer items-center justify-between px-2.5 py-2 text-xs';
	const iconClass = 'shrink-0 text-muted-foreground';

	async function openUrl(url: string) {
		try {
			await openExternalUrl(url);
		} catch (cause) {
			console.error('Unable to open external link.', cause);
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<DropdownMenu dir={direction}>
	<DropdownMenuTrigger>
		{#snippet child({ props }: { props: Record<string, unknown> })}
			<Tooltip>
				<TooltipTrigger>
					<Button
						{...props}
						variant="outline"
						size="icon"
						class="size-6 cursor-pointer rounded-md"
						aria-label={m.sidebar_more_actions()}
					>
						<MoreHorizontal size={13} strokeWidth={1.8} class="text-foreground" />
					</Button>
				</TooltipTrigger>
				<TooltipContent side={tooltipSide} sideOffset={12}>
					{m.sidebar_more_actions()}
				</TooltipContent>
			</Tooltip>
		{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuPortal>
		<DropdownMenuContent
			align="end"
			side={tooltipSide}
			sideOffset={12}
			dir={direction}
			class="z-50 w-64 select-none overflow-visible p-1 text-start"
		>
			<DropdownMenuSub>
				<DropdownMenuSubTrigger class={subTriggerClass}>
					<span class="flex items-center gap-2.5">
						<CircleHelp size={14} class={iconClass} />
						{m.sidebar_resources()}
					</span>
				</DropdownMenuSubTrigger>
				<DropdownMenuSubContent align="end" sideOffset={4} dir={direction} class="w-48 text-start">
					<DropdownMenuItem class={menuItemClass} onclick={() => void openUrl('https://www.mulverse.com/')}>
						<Globe2 size={13} class={iconClass} />
						{m.sidebar_welcome()}
					</DropdownMenuItem>
					<DropdownMenuItem class={menuItemClass} onclick={() => void openUrl('https://docs.orbis.edu')}>
						<FileText size={13} class={iconClass} />
						{m.sidebar_documentation()}
					</DropdownMenuItem>
					<DropdownMenuItem class={menuItemClass} onclick={() => void openUrl('https://docs.orbis.edu/api')}>
						<Code2 size={13} class={iconClass} />
						{m.sidebar_api_docs()}
					</DropdownMenuItem>
					<DropdownMenuItem class={menuItemClass} onclick={() => void openUrl('https://help.orbis.edu')}>
						<CircleHelp size={13} class={iconClass} />
						{m.sidebar_help_center()}
					</DropdownMenuItem>
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://www.mulverse.com/p/orbis/support')}
					>
						<Headphones size={13} class={iconClass} />
						{m.sidebar_contact_support()}
					</DropdownMenuItem>
				</DropdownMenuSubContent>
			</DropdownMenuSub>

			<DropdownMenuSub>
				<DropdownMenuSubTrigger class={subTriggerClass}>
					<span class="flex items-center gap-2.5">
						<UsersRound size={14} class={iconClass} />
						{m.sidebar_community()}
					</span>
				</DropdownMenuSubTrigger>
				<DropdownMenuSubContent align="end" sideOffset={4} dir={direction} class="w-48 text-start">
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://discord.gg/e64gRQZhP')}
					>
						<MessageCircleMore size={13} class={iconClass} />
						Discord
					</DropdownMenuItem>
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://x.com/mulverseco')}
					>
						<span class="text-xs font-semibold">X</span>
						Twitter / X
					</DropdownMenuItem>
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://github.com/mulverseco')}
					>
						<svg
							class="size-3.5 shrink-0 fill-current text-muted-foreground"
							viewBox="0 0 24 24"
							aria-hidden="true"
						>
							<path
								d="M12 .9a11.1 11.1 0 0 0-3.51 21.63c.56.1.76-.24.76-.54v-2.1c-3.1.68-3.76-1.31-3.76-1.31-.51-1.3-1.24-1.65-1.24-1.65-1.02-.7.08-.69.08-.69 1.13.08 1.73 1.16 1.73 1.16 1 .1.75 2.48 3.97 1.75.1-.72.4-1.21.7-1.49-2.47-.28-5.06-1.24-5.06-5.51 0-1.22.44-2.22 1.16-3-.12-.28-.5-1.42.11-2.96 0 0 .95-.3 3.12 1.15a10.85 10.85 0 0 1 5.68 0c2.17-1.46 3.12-1.15 3.12-1.15.62 1.54.23 2.68.11 2.96.72.78 1.16 1.78 1.16 3 0 4.28-2.6 5.23-5.08 5.5.4.35.76 1.02.76 2.06v3.05c0 .3.2.65.77.54A11.1 11.1 0 0 0 12 .9Z"
							/>
						</svg>
						GitHub
					</DropdownMenuItem>
				</DropdownMenuSubContent>
			</DropdownMenuSub>

			<DropdownMenuSub>
				<DropdownMenuSubTrigger class={subTriggerClass}>
					<span class="flex items-center gap-2.5">
						<Scale size={14} class={iconClass} />
						{m.sidebar_legal()}
					</span>
				</DropdownMenuSubTrigger>
				<DropdownMenuSubContent align="end" sideOffset={4} dir={direction} class="w-48 text-start">
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://www.mulverse.com/p/orbis/terms')}
					>
						<FileText size={13} class={iconClass} />
						{m.sidebar_terms()}
					</DropdownMenuItem>
					<DropdownMenuItem
						class={menuItemClass}
						onclick={() => void openUrl('https://www.mulverse.com/p/orbis/privacy')}
					>
						<LockKeyhole size={13} class={iconClass} />
						{m.sidebar_privacy()}
					</DropdownMenuItem>
				</DropdownMenuSubContent>
			</DropdownMenuSub>

			<DropdownMenuSeparator />

			<DropdownMenuItem class={menuItemClass} onclick={() => (isSupportOpen = true)}>
				<MessageCircleMore size={14} class={iconClass} />
				{m.sidebar_feedback()}
			</DropdownMenuItem>

			<DropdownMenuSub>
				<DropdownMenuSubTrigger class={subTriggerClass}>
					<span class="flex items-center gap-2.5">
						<Code2 size={14} class={iconClass} />
						{m.sidebar_system_tools()}
					</span>
				</DropdownMenuSubTrigger>
				<DropdownMenuSubContent align="end" sideOffset={4} dir={direction} class="w-48 text-start">
					<DropdownMenuItem class={menuItemClass} onclick={() => (isSupportOpen = true)}>
						<Headphones size={13} class={iconClass} />
						{m.sidebar_open_support()}
					</DropdownMenuItem>
					<DropdownMenuItem class={menuItemClass} onclick={() => (isAboutOpen = true)}>
						<CircleHelp size={13} class={iconClass} />
						{m.sidebar_app_info()}
					</DropdownMenuItem>
					{#if isAdmin}
						<DropdownMenuItem class={menuItemClass} onclick={() => (isBackupCliOpen = true)}>
							<Code2 size={13} class={iconClass} />
							{m.sidebar_database_tools()}
						</DropdownMenuItem>
					{/if}
				</DropdownMenuSubContent>
			</DropdownMenuSub>

			<DropdownMenuItem
				class={menuItemClass}
				onclick={() => void openUrl('https://www.mulverse.com/')}
			>
				<Globe2 size={14} class={iconClass} />
				{m.sidebar_tour()}
			</DropdownMenuItem>
		</DropdownMenuContent>
	</DropdownMenuPortal>
</DropdownMenu>

<AboutDialog bind:open={isAboutOpen} />
<SupportDialog bind:open={isSupportOpen} />
<BackupCliDialog bind:open={isBackupCliOpen} organizationId={orgId} />
