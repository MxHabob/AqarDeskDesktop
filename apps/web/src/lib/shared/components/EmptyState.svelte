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

<div class="grid min-h-56 content-center justify-items-center gap-3 rounded-lg border border-dashed border-border bg-card p-8 text-center shadow-card">
	<span class="grid size-12 place-items-center rounded-full bg-accent text-accent-foreground">
		<Building2 size={22} />
	</span>
	<h2 class="text-base font-semibold text-foreground">{title}</h2>
	<p class="max-w-md text-sm leading-relaxed text-muted-foreground">{description}</p>
	{#if children}{@render children()}{/if}
</div>
