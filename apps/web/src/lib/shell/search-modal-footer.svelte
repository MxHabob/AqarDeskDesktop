<script lang="ts">
	import * as m from '#lib/paraglide/messages';
</script>

<footer class="flex h-10 shrink-0 items-center justify-between gap-2 border-t border-border/70 bg-muted/30 px-4 text-[10px] text-muted-foreground">
	<span>{m.search_navigate_hint()}</span>
	<div class="flex items-center gap-2">
		<kbd class="rounded border border-border bg-background px-1.5 py-0.5 font-mono">↑ ↓</kbd>
		<kbd class="rounded border border-border bg-background px-1.5 py-0.5 font-mono">Enter</kbd>
		<kbd class="rounded border border-border bg-background px-1.5 py-0.5 font-mono">Esc</kbd>
	</div>
</footer>
