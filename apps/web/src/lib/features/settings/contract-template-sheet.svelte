<script lang="ts">
	import { FilePlus2, Pencil } from '@lucide/svelte';
	import { Button, Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import ContractTemplateForm from '#lib/features/settings/contract-template-form.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		initial?: ContractTemplate;
		busy: boolean;
		onSubmit: (input: ContractTemplateInput) => Promise<boolean>;
		open?: boolean;
		showTrigger?: boolean;
	};

	let { initial, busy, onSubmit, open = $bindable(false), showTrigger = true }: Props = $props();

	function submit(input: ContractTemplateInput) {
		return onSubmit(input);
	}
</script>

{#if showTrigger}
	<Button
		variant={initial ? 'ghost' : 'default'}
		size={initial ? 'icon' : 'default'}
		class={initial ? 'size-8' : 'shrink-0'}
		aria-label={initial ? m.settings_template_edit() : m.settings_template_add()}
		onclick={() => (open = true)}
	>
		{#if initial}
			<Pencil size={15} />
		{:else}
			<FilePlus2 size={16} />
			{m.settings_template_add()}
		{/if}
	</Button>
{/if}

<Sheet bind:open>
	<SheetContent
		side="end"
		class="max-w-none"
		contentClass="!data-[side=end]:w-[min(78rem,calc(100vw-1rem))]"
	>
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{initial ? m.settings_template_edit() : m.settings_template_add()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<ContractTemplateForm
				{initial}
				{busy}
				onSubmit={submit}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
