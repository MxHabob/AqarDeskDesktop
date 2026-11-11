<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Label } from '@aqardesk/ui';

	type Props = {
		label: string;
		id?: string;
		description?: string;
		children: Snippet;
	};

	let { label, id, description, children }: Props = $props();
</script>

<div class="grid min-w-0 gap-2">
	<Label for={id} class="font-medium text-slate-700">{label}</Label>
	{@render children()}
	{#if description}
		<p class="text-xs leading-relaxed text-slate-500">{description}</p>
	{/if}
</div>
