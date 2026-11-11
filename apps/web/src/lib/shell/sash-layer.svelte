<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="relative flex h-full min-h-0 min-w-0 flex-1 overflow-hidden">
	{@render children()}
</div>
