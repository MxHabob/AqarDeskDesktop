<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import TemplateStudio from '#lib/features/templates/template-studio.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		templateId: string;
	};

	let { templateId }: Props = $props();
	const app = getAppContext();
	const templatesPath = $derived(
		`/${encodeURIComponent(app.currentUser?.organization_id ?? '')}/templates`
	);
	let initial = $state<ContractTemplate | undefined>();
	let name = $state('');
	let loading = $state(true);
	let loadError = $state('');

	onMount(() => {
		void loadTemplate();
	});

	async function loadTemplate() {
		try {
			const templates = await desktopApi.listContractTemplates();
			initial = templates.find((template) => template.id === templateId);
			if (!initial) {
				loadError = m.settings_template_not_found();
				return;
			}
			name = initial.name;
		} catch (cause) {
			loadError = cause instanceof Error ? cause.message : String(cause);
			toast.error(loadError);
		} finally {
			loading = false;
		}
	}

	async function save(input: ContractTemplateInput): Promise<ContractTemplate> {
		const saved = await desktopApi.updateContractTemplate(templateId, input);
		initial = saved;
		name = saved.name;
		return saved;
	}

	function returnToList() {
		void goto(templatesPath);
	}
</script>

<svelte:head>
	<title>{name} · {m.settings_contract_templates()}</title>
</svelte:head>

{#if loading}
	<div class="grid h-full min-h-0 place-content-center text-sm text-muted-foreground">
		{m.common_loading()}
	</div>
{:else if loadError}
	<div class="grid h-full min-h-0 place-content-center gap-3 text-center">
		<p class="text-sm text-destructive" role="alert">{loadError}</p>
		<Button variant="outline" onclick={returnToList}>{m.settings_template_back_to_list()}</Button>
	</div>
{:else if initial}
	<TemplateStudio template={initial} onBack={returnToList} onSave={save} />
{/if}
