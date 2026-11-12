<script lang="ts">
	import type { PageProps } from './$types';
	import ContractTemplateEditorPage from '#lib/features/settings/contract-template-editor-page.svelte';

	let { params }: PageProps = $props();
	const templateId = $derived(params.templateId);
</script>

<ContractTemplateEditorPage {templateId} />
