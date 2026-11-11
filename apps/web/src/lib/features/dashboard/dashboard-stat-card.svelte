<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { Building2 } from '@lucide/svelte';
	import { getLocale } from '#lib/paraglide/runtime';

	let {
		label,
		value,
		icon: Icon,
		description = ''
	}: {
		label: string;
		value: number;
		icon: typeof Building2;
		description?: string;
	} = $props();

	const formattedValue = $derived(new Intl.NumberFormat(getLocale()).format(value));
</script>

<Card class="flex h-[166px] min-w-0 flex-col justify-between gap-5 p-3">
	<div class="flex items-center gap-2 text-xs text-muted-foreground">
		<Icon size={14} />
		<span>{label}</span>
	</div>
	<div class="grid gap-1">
		<strong class="text-3xl font-normal tracking-tight text-foreground">{formattedValue}</strong>
		{#if description}<span class="text-[11px] text-muted-foreground">{description}</span>{/if}
	</div>
</Card>
