<script lang="ts">
	import { ArrowUpRight, Search } from '@lucide/svelte';
	import type { SearchItem } from './search-modal.types';
	import * as m from '#lib/paraglide/messages';

	let {
		items,
		selectedIndex,
		onSelect,
		onHover
	}: {
		items: SearchItem[];
		selectedIndex: number;
		onSelect: (item: SearchItem) => void;
		onHover: (index: number) => void;
	} = $props();
</script>

<div class="max-h-[min(60dvh,26rem)] min-h-32 overflow-y-auto p-2 scrollbar-none">
	{#if items.length === 0}
		<div class="flex min-h-32 flex-col items-center justify-center gap-2 text-center">
			<span class="grid size-9 place-items-center rounded-full bg-muted text-muted-foreground">
				<Search size={16} />
			</span>
			<p class="text-sm text-muted-foreground">{m.search_no_results()}</p>
		</div>
	{:else}
		<p class="px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
			{m.nav_workspace()}
		</p>
		<div class="grid gap-1">
			{#each items as item, index (item.key)}
				<button
					type="button"
					aria-current={index === selectedIndex ? 'true' : undefined}
					class={index === selectedIndex
						? 'flex min-h-11 w-full items-center gap-3 rounded-lg bg-accent px-3 text-start text-accent-foreground'
						: 'flex min-h-11 w-full items-center gap-3 rounded-lg px-3 text-start text-foreground transition-colors hover:bg-muted'}
					onclick={() => onSelect(item)}
					onmouseenter={() => onHover(index)}
				>
					<span class="grid size-7 shrink-0 place-items-center rounded-md bg-background text-muted-foreground">
						<Search size={14} />
					</span>
					<span class="min-w-0 flex-1 truncate text-sm">{item.label}</span>
					<ArrowUpRight size={14} class="shrink-0 text-muted-foreground rtl:-rotate-90" />
				</button>
			{/each}
		</div>
	{/if}
</div>
