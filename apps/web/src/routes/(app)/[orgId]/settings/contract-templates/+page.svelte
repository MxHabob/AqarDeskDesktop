<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '@aqardesk/ui';
	import {
		AqarDeskGrid,
		type GridActionEvent,
		type AqarDeskColumnDef
	} from '@aqardesk/svelte-grid';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import ContractTemplateSheet from '#lib/features/settings/contract-template-sheet.svelte';
	import * as m from '#lib/paraglide/messages';

	let templates = $state<ContractTemplate[]>([]);
	const app = getAppContext();
	let loading = $state(true);
	let busy = $state(false);
	let selectedTemplate = $state<ContractTemplate | null>(null);
	let editSheetOpen = $state(false);
	const rtl = $derived(getLocale() === 'ar');
	function hasTemplateContent(row: Record<string, unknown>) {
		const latest = row.versions instanceof Array ? row.versions[0] : undefined;
		return typeof latest?.content_body === 'string' && latest.content_body.trim().length > 0;
	}

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'name', header: m.settings_template_name(), width: 200 },
		{ id: 'template_key', header: m.settings_template_key(), width: 200 },
		{ id: 'description', header: m.settings_template_description(), width: 240 },
		{
			id: 'lease_type',
			header: m.settings_template_lease_type(),
			type: 'enum',
			width: 150,
			options: [
				{ value: 'residential', label: m.properties_type_residential() },
				{ value: 'commercial', label: m.properties_type_commercial() },
				{ value: 'short_term', label: m.settings_template_short_term() },
				{ value: 'other', label: m.common_other() },
				{ value: 'receipt', label: m.settings_template_receipt() }
			]
		},
		{ id: 'locale', header: m.settings_template_locale(), width: 100 },
		{
			id: 'versions',
			header: m.settings_template_versions(),
			width: 360,
			accessor: (row) => JSON.stringify(row.versions ?? [])
		},
		{
			id: 'status',
			header: m.users_status(),
			type: 'status',
			width: 120,
			accessor: (row) =>
				row.is_active
					? 'active'
					: row.versions instanceof Array && row.versions[0]?.content_body?.trim()
						? 'archived'
						: 'draft',
			options: [
				{ value: 'active', label: m.settings_template_active() },
				{ value: 'draft', label: m.settings_template_draft() },
				{ value: 'archived', label: m.settings_template_archived() }
			]
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{ id: 'updated_at', header: m.settings_updated_at(), type: 'datetime', width: 190 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'edit',
						icon: 'Pencil',
						label: m.settings_template_edit(),
						variant: 'outline',
						disabled: () => busy
					},
					{
						id: 'disable',
						icon: 'ArchiveBox',
						label: m.settings_template_disable(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active === true,
						disabled: () => busy
					},
					{
						id: 'enable',
						icon: 'Check',
						label: (row: Record<string, unknown>) =>
							hasTemplateContent(row)
								? m.settings_template_enable()
								: m.settings_template_enable_content_first(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active !== true,
						disabled: (row: Record<string, unknown>) => busy || !hasTemplateContent(row)
					}
				]
			}
		}
	]);

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

	function handleGridAction(event: GridActionEvent) {
		const template = templates.find((item) => item.id === String(event.row.id ?? ''));
		if (!template) return;
		if (event.actionId === 'dblclick') {
			selectedTemplate = template;
			editSheetOpen = true;
		} else if (event.actionId === 'edit') {
			selectedTemplate = template;
			editSheetOpen = true;
		} else if (event.actionId === 'enable' || event.actionId === 'disable') {
			void changeActive(template);
		}
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={templates}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		rowIdKey="id"
		height="100%"
		emptyTitle={m.settings_template_empty()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<ContractTemplateSheet
				busy={busy}
				onSubmit={(input) => save(input)}
			/>
		{/snippet}
	</AqarDeskGrid>
</div>

{#if selectedTemplate}
	<ContractTemplateSheet
		initial={selectedTemplate}
		bind:open={editSheetOpen}
		showTrigger={false}
		{busy}
		onSubmit={(input) => save(input, selectedTemplate ?? undefined)}
	/>
{/if}
