<script lang="ts">
	type Props = {
		label: string;
		active?: boolean;
	};

	let { label, active = true }: Props = $props();
</script>

<span
	class="inline-flex items-center gap-2 rounded-full px-2.5 py-1 text-xs font-medium"
	class:bg-emerald-50={active}
	class:text-emerald-800={active}
	class:bg-slate-100={!active}
	class:text-slate-600={!active}
>
	<span
		class="size-1.5 rounded-full"
		class:bg-emerald-600={active}
		class:bg-slate-400={!active}
	></span>
	{label}
</span>
