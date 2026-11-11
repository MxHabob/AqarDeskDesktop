<script lang="ts">
	import { Coins, Pencil } from '@lucide/svelte';
	import { Button, Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { CurrencySettings } from '@aqardesk/sdk';
	import type { CurrencyEditorInput } from '#lib/api/desktop';
	import CurrencyForm from '#lib/features/settings/currency-form.svelte';
	import * as m from '#lib/paraglide/messages';

	let {
		initial,
		busy,
		onSubmit,
		open = $bindable(false),
		showTrigger = true
	}: {
		initial?: CurrencySettings;
		busy: boolean;
		onSubmit: (input: CurrencyEditorInput) => Promise<boolean>;
		open?: boolean;
		showTrigger?: boolean;
	} = $props();
</script>

{#if showTrigger}
	<Button
		variant={initial ? 'ghost' : 'default'}
		size={initial ? 'icon' : 'default'}
		class={initial ? 'size-8' : 'shrink-0'}
		aria-label={initial ? m.settings_currency_edit() : m.settings_currency_add()}
		onclick={() => (open = true)}
	>
		{#if initial}
			<Pencil size={15} />
		{:else}
			<Coins size={16} />
			{m.settings_currency_add()}
		{/if}
	</Button>
{/if}

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{initial ? m.settings_currency_edit() : m.settings_currency_add()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<CurrencyForm
				{initial}
				{busy}
				{onSubmit}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
