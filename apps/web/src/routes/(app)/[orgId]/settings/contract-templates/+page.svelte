<script lang="ts">
	import { onMount } from 'svelte';
	import { Check, ChevronDown, FileText } from '@lucide/svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import ContractTemplateSheet from '#lib/features/settings/contract-template-sheet.svelte';
	import * as m from '#lib/paraglide/messages';

	let templates = $state<ContractTemplate[]>([]);
	let loading = $state(true);
	let busy = $state(false);

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			templates = await desktopApi.listContractTemplates();
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function save(input: ContractTemplateInput, initial?: ContractTemplate) {
		busy = true;
		try {
			if (initial) await desktopApi.updateContractTemplate(initial.id, input);
			else await desktopApi.createContractTemplate(input);
			await load();
			toast.success(m.common_save());
			return true;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
			return false;
		} finally {
			busy = false;
		}
	}

	async function changeActive(template: ContractTemplate) {
		const nextActive = !template.is_active;
		const confirmed = await confirmStore.prompt({
			title: nextActive ? m.settings_template_enable() : m.settings_template_disable(),
			description: template.name,
			confirmText: nextActive ? m.settings_template_enable() : m.settings_template_disable(),
			cancelText: m.common_cancel(),
			variant: nextActive ? 'default' : 'destructive'
		});
		if (!confirmed) return;
		busy = true;
		try {
			await desktopApi.setContractTemplateActive(template.id, nextActive);
			await load();
			toast.success(nextActive ? m.settings_template_enable() : m.settings_template_disable());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			busy = false;
		}
	}
</script>

<header class="flex items-start justify-between gap-4">
	<div class="grid gap-1">
		<h2 class="text-lg font-semibold">{m.settings_contract_templates()}</h2>
		<p class="text-sm text-muted-foreground">{m.settings_contract_templates_description()}</p>
	</div>
	<ContractTemplateSheet busy={busy} onSubmit={(input) => save(input)} />
</header>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if templates.length === 0}
	<p class="py-8 text-sm text-muted-foreground">{m.settings_template_empty()}</p>
{:else}
	<div class="grid divide-y divide-border/70 border-y border-border/70">
		{#each templates as template (template.id)}
			{@const latest = template.versions[0]}
			<article class="grid gap-3 py-4">
				<div class="flex min-w-0 items-start gap-3">
					<span class="mt-0.5 grid size-9 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
						<FileText size={17} />
					</span>
					<div class="grid min-w-0 flex-1 gap-1">
						<div class="flex flex-wrap items-center gap-2">
							<h3 class="text-sm font-semibold">{template.name}</h3>
							<span class="rounded-full bg-muted px-2 py-0.5 text-[10px] text-muted-foreground">
								{template.is_active ? m.settings_template_active() : m.settings_template_archived()}
							</span>
						</div>
						<p class="truncate text-xs text-muted-foreground" dir="ltr">
							{template.template_key} · {template.locale} · {template.lease_type}
						</p>
						{#if template.description}
							<p class="text-xs text-muted-foreground">{template.description}</p>
						{/if}
					</div>
					{#if template.is_active}
						<ContractTemplateSheet
							initial={template}
							{busy}
							onSubmit={(input) => save(input, template)}
						/>
					{/if}
					<Button
						variant="ghost"
						size="icon"
						class="size-8"
						aria-label={template.is_active ? m.settings_template_disable() : m.settings_template_enable()}
						disabled={busy}
						onclick={() => void changeActive(template)}
					>
						{#if template.is_active}<Check size={15} />{:else}<ChevronDown size={15} />{/if}
					</Button>
				</div>
				{#if latest}
					<details class="ms-12 rounded-md border border-border/70 px-3 py-2">
						<summary class="cursor-pointer text-xs font-medium text-muted-foreground">
							{m.settings_template_version()} {latest.version_number} · {latest.content_format}
						</summary>
						<div class="mt-3 grid gap-2">
							<p class="text-xs text-muted-foreground" dir="ltr">{latest.content_sha256}</p>
							{#if latest.change_note}
								<p class="text-xs">{latest.change_note}</p>
							{/if}
							<pre class="max-h-64 overflow-auto whitespace-pre-wrap rounded-md bg-muted/50 p-3 text-xs" dir="auto">{latest.content_body}</pre>
						</div>
					</details>
				{/if}
			</article>
		{/each}
	</div>
{/if}
