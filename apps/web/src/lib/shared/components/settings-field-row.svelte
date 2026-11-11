<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Copy } from '@lucide/svelte';
	import { Button, Label, toast } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		label: string;
		id?: string;
		description?: string;
		children?: Snippet;
		action?: Snippet;
		copyValue?: string;
	};

	let { label, id, description, children, action, copyValue }: Props = $props();

	async function copyFieldValue() {
		if (!copyValue) return;
		try {
			await navigator.clipboard.writeText(copyValue);
			toast.success(m.settings_copy_success());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<div class="grid min-w-0 gap-2 px-4 py-4 sm:grid-cols-[15rem_minmax(0,1fr)_auto] sm:items-center sm:gap-6">
	<div class="grid gap-1">
		{#if id}
			<Label for={id} class="text-sm font-medium">{label}</Label>
		{:else}
			<p class="text-sm font-medium">{label}</p>
		{/if}
		{#if description}
			<p class="text-xs text-muted-foreground">{description}</p>
		{/if}
	</div>
	{#if children}
		<div class={action || copyValue ? 'min-w-0' : 'min-w-0 sm:col-span-2'}>
			{@render children()}
		</div>
	{:else if action || copyValue}
		<div class="hidden sm:block sm:col-span-1"></div>
	{/if}
	{#if action}
		<div class="justify-self-start">
			{@render action()}
		</div>
	{:else if copyValue}
		<Button
			type="button"
			variant="outline"
			class="shrink-0"
			onclick={() => void copyFieldValue()}
		>
			<Copy size={15} />
			{m.settings_copy()}
		</Button>
	{/if}
</div>
