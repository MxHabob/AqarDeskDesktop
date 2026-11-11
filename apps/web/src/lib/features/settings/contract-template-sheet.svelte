<script lang="ts">
	import { tick } from 'svelte';
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
	let name = $state('');
	let editingName = $state(false);
	let nameInput = $state<HTMLInputElement>();

	$effect(() => {
		if (open) {
			name = initial?.name ?? '';
			editingName = false;
		}
	});

	async function editName() {
		if (editingName) return;
		editingName = true;
		await tick();
		nameInput?.focus();
		nameInput?.select();
	}

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
		class="!p-0 max-w-none"
		contentClass="!data-[side=end]:end-0 !data-[side=end]:right-0 !data-[side=end]:h-dvh !data-[side=end]:w-screen"
	>
		<SheetHeader class="shrink-0 px-4 py-4 pe-12">
			<SheetTitle ondblclick={editName} class="min-h-7 cursor-text">
				{#if editingName}
					<input
						bind:this={nameInput}
						bind:value={name}
						aria-label={m.settings_template_name()}
						onblur={() => (editingName = false)}
						onkeydown={(event) => {
							if (event.key === 'Enter') {
								event.preventDefault();
								editingName = false;
							}
						}}
						class="w-full border-0 bg-transparent p-0 text-base font-semibold leading-none outline-none focus-visible:ring-0"
					/>
				{:else}
					<span>{name || m.settings_template_new()}</span>
				{/if}
			</SheetTitle>
			<p class="text-xs text-muted-foreground">{m.settings_template_rename_hint()}</p>
		</SheetHeader>
		{#if open}
			<ContractTemplateForm
				{initial}
				bind:name
				{busy}
				onSubmit={submit}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
