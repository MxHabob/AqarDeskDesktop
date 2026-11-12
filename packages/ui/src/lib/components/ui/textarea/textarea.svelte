<script lang="ts">
	import { cn, type WithElementRef } from "@aqardesk/ui/utils.js";
	import type { HTMLTextareaAttributes } from "svelte/elements";

	type Props = WithElementRef<HTMLTextareaAttributes>;

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		"data-slot": dataSlot = "textarea",
		...restProps
	}: Props = $props();
</script>

<textarea
	bind:this={ref}
	bind:value
	data-slot={dataSlot}
	class={cn(
		"border-input bg-card text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 aria-invalid:border-destructive disabled:bg-muted min-h-20 w-full rounded-md border px-3 py-2 text-sm outline-none transition-colors focus-visible:ring-2 aria-invalid:ring-1 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60",
		className
	)}
	{...restProps}
></textarea>
