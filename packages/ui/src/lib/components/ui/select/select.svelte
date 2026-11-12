<script lang="ts">
	import { Select as SelectPrimitive } from "bits-ui";

	let {
		open = $bindable(false),
		value = $bindable(),
		items,
		...restProps
	}: SelectPrimitive.RootProps & {
		items: NonNullable<SelectPrimitive.RootProps['items']>;
	} = $props();
</script>

<SelectPrimitive.Root bind:open bind:value={value as never} {items} {...restProps} />
