<script lang="ts">
	import * as m from '#lib/paraglide/messages';

	type MenuItem = { key: string; label: string };

	let {
		items,
		active,
		onSelect
	}: {
		items: MenuItem[];
		active: string;
		onSelect: (key: string) => void;
	} = $props();
</script>

<nav
	class="flex h-11 shrink-0 items-center gap-1 overflow-x-auto border-b border-border/70 bg-background px-4 scrollbar-none sm:px-6"
	aria-label={m.nav_workspace()}
>
	{#each items as item (item.key)}
		<button
			type="button"
			aria-current={active === item.key ? 'page' : undefined}
			class={active === item.key
				? 'relative h-full shrink-0 px-3 text-xs font-semibold text-foreground after:absolute after:inset-x-3 after:bottom-0 after:h-0.5 after:rounded-full after:bg-primary'
				: 'h-full shrink-0 px-3 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground'}
			onclick={() => onSelect(item.key)}
		>
			{item.label}
		</button>
	{/each}
</nav>
