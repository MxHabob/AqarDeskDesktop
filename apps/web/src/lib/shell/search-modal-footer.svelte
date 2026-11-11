<script lang="ts">
	import logo from '#lib/assets/aqardesk-logo.svg';
	import * as m from '#lib/paraglide/messages';
</script>

<footer class="flex h-10 shrink-0 items-center justify-between gap-2 border-t border-border bg-background px-3 text-[10px] text-muted-foreground">
	<img src={logo} class="ms-0 size-4 opacity-70" alt="" aria-hidden="true" />
	<div class="ms-auto flex gap-x-2" role="group" aria-label={m.search_navigate_hint()}>
		<kbd class="flex size-6 select-none items-center justify-center border bg-accent font-mono">↑</kbd>
		<kbd class="flex size-6 select-none items-center justify-center border bg-accent font-mono">↓</kbd>
		<kbd class="flex size-6 select-none items-center justify-center border bg-accent font-mono">↵</kbd>
	</div>
</footer>
