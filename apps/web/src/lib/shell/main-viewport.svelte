<script lang="ts">
	import type { Snippet } from 'svelte';
	import Resizer from './resizer.svelte';
	import SashLayer from './sash-layer.svelte';

	let {
		name,
		left,
		middle,
		sidebarSize = $bindable(2),
		showLeft = true
	}: {
		name: string;
		left: Snippet;
		middle: Snippet;
		sidebarSize?: number;
		showLeft?: boolean;
	} = $props();

	const sidebarWidth = $derived(
		(['w-14', 'w-48', 'w-60', 'w-72'] as const)[sidebarSize]
	);
</script>

<SashLayer>
	<div class="flex h-full min-h-0 min-w-0 flex-1 overflow-hidden">
		{#if showLeft}
			<div class={`relative hidden h-full min-h-0 shrink-0 flex-col overflow-hidden md:flex ${sidebarWidth}`}>
				{@render left()}
			</div>
			<div class="hidden h-full md:flex">
				<Resizer
					value={sidebarSize}
					persistId={name}
					onChange={(width) => (sidebarSize = width)}
				/>
			</div>
		{/if}
		<div class="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
			{@render middle()}
		</div>
	</div>
</SashLayer>
