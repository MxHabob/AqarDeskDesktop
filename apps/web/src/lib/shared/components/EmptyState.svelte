<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Building2 } from '@lucide/svelte';

	type Props = {
		title: string;
		description: string;
		children?: Snippet;
	};

	let { title, description, children }: Props = $props();
</script>

<div class="grid min-h-56 content-center justify-items-center gap-3 rounded-lg border border-dashed border-slate-300 bg-white p-8 text-center">
	<span class="grid size-12 place-items-center rounded-full bg-emerald-50 text-emerald-800">
		<Building2 size={22} />
	</span>
	<h2 class="text-base font-semibold text-slate-900">{title}</h2>
	<p class="max-w-md text-sm leading-relaxed text-slate-600">{description}</p>
	{#if children}{@render children()}{/if}
</div>
