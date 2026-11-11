<script lang="ts">
	type Props = {
		label: string;
		active?: boolean;
	};

	let { label, active = true }: Props = $props();
</script>

<span
	class={`inline-flex items-center gap-2 rounded-full px-2.5 py-1 text-xs font-medium ${
		active ? 'bg-primary/20 text-foreground' : 'bg-muted text-muted-foreground'
	}`}
>
	<span
		class="size-1.5 rounded-full"
		class:bg-primary-foreground={active}
		class:bg-muted-foreground={!active}
	></span>
	{label}
</span>
