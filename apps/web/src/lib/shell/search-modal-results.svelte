<script lang="ts">
	import { tick } from 'svelte';
	import {
		ArrowUpRight,
		Building2,
		LayoutDashboard,
		ReceiptText,
		ScrollText,
		Search,
		Settings2,
		UsersRound
	} from '@lucide/svelte';
	import type { SearchItem } from './search-modal.types';
	import * as m from '#lib/paraglide/messages';

	const itemIcons: Partial<Record<SearchItem['key'], typeof Building2>> = {
		overview: LayoutDashboard,
		properties: Building2,
		units: Building2,
		tenants: UsersRound,
		owners: UsersRound,
		contracts: ScrollText,
		payments: ReceiptText,
		users: UsersRound,
		settings: Settings2
	};

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

	let listElement = $state<HTMLDivElement | null>(null);

	$effect(() => {
		if (!items[selectedIndex]) return;
		void tick().then(() => {
			listElement
				?.querySelector('[aria-selected="true"]')
				?.scrollIntoView({ block: 'nearest' });
		});
	});
</script>

<div
	bind:this={listElement}
	id="global-search-results"
	role="listbox"
	aria-label={m.nav_workspace()}
	class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-2 scrollbar-none"
>
	{#if items.length === 0}
		<div class="flex h-full min-h-32 flex-col items-center justify-center gap-2 text-center">
			<span class="grid size-9 place-items-center rounded-full bg-muted text-muted-foreground">
				<Search size={16} />
			</span>
			<p class="text-sm text-muted-foreground">{m.search_no_results()}</p>
		</div>
	{:else}
		<p
			id="search-results-heading"
			class="px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground"
		>
			{m.nav_workspace()}
		</p>
		<div role="group" aria-labelledby="search-results-heading" class="grid gap-1">
			{#each items as item, index (item.key)}
				{@const Icon = itemIcons[item.key] ?? Search}
				<button
					type="button"
					id={`search-result-${item.key}`}
					role="option"
					aria-selected={index === selectedIndex}
					tabindex="-1"
					class={index === selectedIndex
						? 'flex min-h-11 w-full items-center gap-3 rounded-lg bg-accent px-3 text-start text-accent-foreground'
						: 'flex min-h-11 w-full items-center gap-3 rounded-lg px-3 text-start text-foreground transition-colors hover:bg-muted'}
					onclick={() => onSelect(item)}
					onmouseenter={() => onHover(index)}
				>
					<span class="grid size-7 shrink-0 place-items-center rounded-md bg-background text-muted-foreground">
						<Icon size={14} />
					</span>
					<span class="min-w-0 flex-1 truncate text-sm">{item.label}</span>
					<ArrowUpRight size={14} class="shrink-0 text-muted-foreground rtl:-rotate-90" />
				</button>
			{/each}
		</div>
	{/if}
</div>
