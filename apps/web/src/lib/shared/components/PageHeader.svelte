<script lang="ts">
	type Props = {
		title: string;
		description: string;
		eyebrow?: string;
	};

	let { title, description, eyebrow }: Props = $props();
</script>

<header class="mb-6 grid gap-1">
	{#if eyebrow}<p class="text-xs font-semibold tracking-wide text-emerald-800">{eyebrow}</p>{/if}
	<h1 class="text-2xl font-semibold tracking-tight text-slate-900">{title}</h1>
	<p class="text-sm leading-relaxed text-slate-600">{description}</p>
</header>
