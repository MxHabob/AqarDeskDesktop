<script lang="ts">
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import { Info } from '@lucide/svelte';

	type Props = {
		title: string;
		description: string;
	};

	let { title, description }: Props = $props();
</script>

<Card class="rounded-lg bg-card p-5 shadow-card ring-border/50">
	<CardHeader class="flex-row items-start gap-3 p-0">
		<span class="grid size-9 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
			<Info size={18} />
		</span>
		<div class="grid gap-1">
			<CardTitle class="text-base">{title}</CardTitle>
			<CardDescription class="text-sm leading-relaxed">{description}</CardDescription>
		</div>
	</CardHeader>
</Card>
