<script lang="ts" module>
	export type SheetSide = 'top' | 'right' | 'bottom' | 'left' | 'start' | 'end';
</script>

<script lang="ts">
	import { Dialog as SheetPrimitive } from 'bits-ui';
	import { X } from '@lucide/svelte';
	import { Button } from '../button/index.js';
	import { cn, type WithoutChildrenOrChild } from '@aqardesk/ui/utils.js';
	import SheetOverlay from './sheet-overlay.svelte';
	import type { ComponentProps, Snippet } from 'svelte';

	let {
		ref = $bindable(null),
		class: className,
		side = 'end',
		showCloseButton = true,
		contentClass,
		portalProps,
		children,
		...restProps
	}: WithoutChildrenOrChild<SheetPrimitive.ContentProps> & {
		portalProps?: WithoutChildrenOrChild<SheetPrimitive.PortalProps>;
		side?: SheetSide;
		showCloseButton?: boolean;
		contentClass?: string;
		children: Snippet;
	} = $props();
</script>

<SheetPrimitive.Portal {...portalProps}>
	<SheetOverlay />
	<SheetPrimitive.Content
		bind:ref
		data-slot="sheet-content"
		data-side={side}
		class={cn(
			'fixed z-[var(--layer-sheet)] flex min-h-0 p-3 transition duration-200 ease-out data-open:animate-in data-closed:animate-out data-open:duration-300 data-closed:duration-200 data-[side=bottom]:inset-x-3 data-[side=bottom]:bottom-3 data-[side=bottom]:w-auto data-[side=bottom]:max-h-[88dvh] data-[side=bottom]:data-open:slide-in-from-bottom data-[side=bottom]:data-closed:slide-out-to-bottom data-[side=left]:inset-y-3 data-[side=left]:left-3 data-[side=left]:h-[calc(100dvh-1.5rem)] data-[side=left]:w-[min(34rem,calc(100vw-1.5rem))] data-[side=left]:data-open:slide-in-from-left data-[side=left]:data-closed:slide-out-to-left data-[side=right]:inset-y-3 data-[side=right]:right-3 data-[side=right]:h-[calc(100dvh-1.5rem)] data-[side=right]:w-[min(34rem,calc(100vw-1.5rem))] data-[side=right]:data-open:slide-in-from-right data-[side=right]:data-closed:slide-out-to-right data-[side=start]:inset-y-3 data-[side=start]:start-3 data-[side=start]:h-[calc(100dvh-1.5rem)] data-[side=start]:w-[min(34rem,calc(100vw-1.5rem))] data-[side=start]:data-open:slide-in-from-left data-[side=start]:data-closed:slide-out-to-left rtl:data-[side=start]:data-open:slide-in-from-right rtl:data-[side=start]:data-closed:slide-out-to-right data-[side=end]:inset-y-3 data-[side=end]:end-3 data-[side=end]:h-[calc(100dvh-1.5rem)] data-[side=end]:w-[min(34rem,calc(100vw-1.5rem))] data-[side=end]:data-open:slide-in-from-right data-[side=end]:data-closed:slide-out-to-right rtl:data-[side=end]:data-open:slide-in-from-left rtl:data-[side=end]:data-closed:slide-out-to-left data-[side=top]:inset-x-3 data-[side=top]:top-3 data-[side=top]:w-auto data-[side=top]:max-h-[88dvh] data-[side=top]:data-open:slide-in-from-top data-[side=top]:data-closed:slide-out-to-top sm:p-4 sm:data-[side=bottom]:inset-x-4 sm:data-[side=bottom]:bottom-4 sm:data-[side=left]:inset-y-4 sm:data-[side=left]:left-4 sm:data-[side=left]:h-[calc(100dvh-2rem)] sm:data-[side=right]:inset-y-4 sm:data-[side=right]:right-4 sm:data-[side=right]:h-[calc(100dvh-2rem)] sm:data-[side=start]:inset-y-4 sm:data-[side=start]:start-4 sm:data-[side=start]:h-[calc(100dvh-2rem)] sm:data-[side=end]:inset-y-4 sm:data-[side=end]:end-4 sm:data-[side=end]:h-[calc(100dvh-2rem)] sm:data-[side=top]:inset-x-4 sm:data-[side=top]:top-4',
			contentClass
		)}
		{...restProps}
	>
		<div
			class={cn(
				'relative flex h-full max-h-full min-h-0 w-full flex-col overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-floating ring-1 ring-foreground/5',
				'data-[side=bottom]:max-h-[88dvh]',
				className
			)}
		>
			{@render children?.()}
			{#if showCloseButton}
				<SheetPrimitive.Close data-slot="sheet-close">
					{#snippet child({ props })}
						<Button variant="ghost" class="absolute end-3 top-3 z-10 bg-secondary" size="icon-sm" {...props}>
							<X />
							<span class="sr-only">Close</span>
						</Button>
					{/snippet}
				</SheetPrimitive.Close>
			{/if}
		</div>
	</SheetPrimitive.Content>
</SheetPrimitive.Portal>
