<script lang="ts">
	import { Popover } from 'bits-ui';
	import { GitMerge } from '@lucide/svelte';
	import { Button, Tooltip, TooltipContent, TooltipTrigger } from '@aqardesk/ui';
	import { openExternalUrl } from '#lib/core/external-url';
	import * as m from '#lib/paraglide/messages';
	import { getHtmlDirection, getInlineEndSide } from '#lib/core/i18n/direction';
	import { toast } from '@aqardesk/ui';

	let { orgId: _orgId }: { orgId: string } = $props();
	let open = $state(false);
	const direction = $derived(getHtmlDirection());
	const tooltipSide = $derived(getInlineEndSide(direction));

	async function visitMulverse() {
		try {
			await openExternalUrl('https://www.mulverse.com/');
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props }: { props: Record<string, unknown> })}
			<Tooltip>
				<TooltipTrigger>
					<Button
						{...props}
						variant="outline"
						size="icon-xs"
						class="relative cursor-pointer"
						aria-label={m.sidebar_mulverse_title()}
					>
						<GitMerge size={13} strokeWidth={1.8} class="text-foreground" />
						<span class="absolute -end-0.5 -top-0.5 size-1 rounded-full bg-indigo-500"></span>
					</Button>
				</TooltipTrigger>
				<TooltipContent side={tooltipSide} sideOffset={12}>
					{m.sidebar_mulverse_title()}
				</TooltipContent>
			</Tooltip>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content
		side={tooltipSide}
		align="end"
		sideOffset={12}
		dir={direction}
		class="z-[var(--layer-popover)] flex max-h-[min(40rem,calc(100vh-4rem))] w-[min(320px,calc(100vw-1rem))] flex-col gap-0 overflow-hidden rounded-xl border border-border bg-popover p-0 text-start text-popover-foreground shadow-floating"
	>
		<div class="border-b border-border px-4 py-3">
			<h2 class="text-sm font-semibold">{m.sidebar_mulverse_title()}</h2>
		</div>
		<p class="px-4 py-3 text-xs leading-relaxed text-muted-foreground">
			{m.sidebar_mulverse_description()}
		</p>
		<div class="border-t border-border px-4 py-3">
			<Button size="sm" class="w-full text-xs font-semibold" onclick={() => void visitMulverse()}>
				{m.sidebar_mulverse_visit()}
			</Button>
		</div>
	</Popover.Content>
</Popover.Root>
