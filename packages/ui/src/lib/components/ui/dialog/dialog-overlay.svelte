<script lang="ts">
	import { Dialog as DialogPrimitive } from "bits-ui";
	import { cn } from "@aqardesk/ui/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: DialogPrimitive.OverlayProps = $props();
</script>

<DialogPrimitive.Overlay
	bind:ref
	data-slot="dialog-overlay"
	class={cn("fixed inset-0 isolate z-[var(--layer-dialog)] bg-overlay/25 duration-75 data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0", className)}
	{...restProps}
/>
