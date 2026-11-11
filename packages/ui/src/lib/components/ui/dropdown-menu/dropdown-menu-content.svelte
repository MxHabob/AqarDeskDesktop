<script lang="ts">
	import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "@aqardesk/ui/utils.js";
	import DropdownMenuPortal from "./dropdown-menu-portal.svelte";
	import type { ComponentProps } from "svelte";

	let {
		ref = $bindable(null),
		sideOffset = 4,
		align = "start",
		portalProps,
		class: className,
		...restProps
	}: DropdownMenuPrimitive.ContentProps & {
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof DropdownMenuPortal>>;
	} = $props();
</script>

<DropdownMenuPortal {...portalProps}>
	<DropdownMenuPrimitive.Content
		bind:ref
		data-slot="dropdown-menu-content"
		{sideOffset}
		{align}
		class={cn(
			"data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 ring-foreground/10 text-popover-foreground min-w-32 rounded-lg border border-border bg-popover shadow-floating ring-1 duration-75 z-[var(--layer-popover)] max-h-(--bits-dropdown-menu-content-available-height) w-(--bits-dropdown-menu-anchor-width) origin-(--bits-dropdown-menu-content-transform-origin) overflow-x-hidden overflow-y-auto **:data-[slot$=-item]:focus:bg-accent **:data-[slot$=-item]:data-highlighted:bg-accent **:data-[slot$=-separator]:bg-border **:data-[slot$=-trigger]:focus:bg-accent **:data-[slot$=-trigger]:aria-expanded:bg-accent",
			className
		)}
		{...restProps}
	/>
</DropdownMenuPortal>
