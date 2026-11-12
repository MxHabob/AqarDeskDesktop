<script lang="ts">
	import { goto } from '$app/navigation';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { FilePlus2 } from '@lucide/svelte';
	import {
		Button,
		Dialog,
		DialogContent,
		DialogDescription,
		DialogHeader,
		DialogTitle,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		toast
	} from '@aqardesk/ui';
	import {
		AqarDeskGrid,
		type GridActionEvent,
		type AqarDeskColumnDef
	} from '@aqardesk/svelte-grid';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { queryClient } from '#lib/core/query-client';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	const pageSize = 50;
	const templatesQuery = createInfiniteQuery(() => ({
		queryKey: ['contract-templates', app.currentUser?.organization_id ?? ''],
		queryFn: ({ pageParam }) => desktopApi.listContractTemplatesPage(pageSize, pageParam),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser
	}));
	const templates = $derived(templatesQuery.data?.pages.flatMap((page) => page.items) ?? []);
	const loading = $derived(templatesQuery.isLoading);
	let busy = $state(false);
	let creating = $state(false);
	let createDialogOpen = $state(false);
	let createDraft = $state({
		name: '',
		description: '',
		lease_type: 'residential',
		locale: 'ar'
	});
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

	$effect(() => {
		if (templatesQuery.error) toast.error(String(templatesQuery.error));
	});

	async function load() {
		await queryClient.invalidateQueries({
			queryKey: ['contract-templates', app.currentUser?.organization_id ?? '']
		});
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
		if (event.actionId === 'select' || event.actionId === 'dblclick' || event.actionId === 'edit') {
			void goto(
				`/${encodeURIComponent(app.currentUser!.organization_id)}/templates/${encodeURIComponent(template.id)}`
			);
		} else if (event.actionId === 'enable' || event.actionId === 'disable') {
			void changeActive(template);
		}
	}

	function createTemplate() {
		createDraft = { name: '', description: '', lease_type: 'residential', locale: getLocale() };
		createDialogOpen = true;
	}

	async function submitCreateTemplate(event: SubmitEvent) {
		event.preventDefault();
		const name = createDraft.name.trim();
		const locale = createDraft.locale.trim();
		if (!name || !locale) return;

		creating = true;
		try {
			const input: ContractTemplateInput = {
				template_key: '',
				name,
				description: createDraft.description.trim() || null,
				lease_type: createDraft.lease_type,
				locale,
				content_format: 'markdown',
				content_body: '',
				placeholders_json: '{}',
				change_note: null
			};
			const template = await desktopApi.createContractTemplate(input);
			createDialogOpen = false;
			await load();
			await goto(
				`/${encodeURIComponent(app.currentUser!.organization_id)}/templates/${encodeURIComponent(template.id)}`
			);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			creating = false;
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
		hasMore={templatesQuery.hasNextPage}
		onloadmore={() => fetchNextPageOrThrow(() => templatesQuery.fetchNextPage())}
		emptyTitle={m.settings_template_empty()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<Button onclick={createTemplate} disabled={busy}>
				<FilePlus2 size={16} class="me-2" />
				{m.settings_template_add()}
			</Button>
		{/snippet}
	</AqarDeskGrid>
</div>

<Dialog bind:open={createDialogOpen}>
	<DialogContent class="max-w-lg">
		<DialogHeader>
			<DialogTitle>{m.settings_template_add()}</DialogTitle>
			<DialogDescription>{m.settings_template_create_dialog_description()}</DialogDescription>
		</DialogHeader>
		<form class="grid gap-4" onsubmit={submitCreateTemplate}>
			<label class="grid gap-1.5 text-sm font-medium">
				{m.settings_template_name()}
				<Input bind:value={createDraft.name} required maxlength={100} autofocus />
			</label>
			<label class="grid gap-1.5 text-sm font-medium">
				{m.settings_template_description()}
				<Input bind:value={createDraft.description} maxlength={500} />
			</label>
			<label class="grid gap-1.5 text-sm font-medium">
				{m.settings_template_lease_type()}
				<Select
					type="single"
					value={createDraft.lease_type}
					items={[
						{ value: 'residential', label: m.properties_type_residential() },
						{ value: 'commercial', label: m.properties_type_commercial() },
						{ value: 'short_term', label: m.settings_template_short_term() },
						{ value: 'other', label: m.common_other() },
						{ value: 'receipt', label: m.settings_template_receipt() }
					]}
					onValueChange={(value) => {
						if (value) createDraft.lease_type = value;
					}}
				>
					<SelectTrigger class="h-9 w-full"><SelectValue /></SelectTrigger>
					<SelectContent>
						<SelectItem value="residential" label={m.properties_type_residential()}>{m.properties_type_residential()}</SelectItem>
						<SelectItem value="commercial" label={m.properties_type_commercial()}>{m.properties_type_commercial()}</SelectItem>
						<SelectItem value="short_term" label={m.settings_template_short_term()}>{m.settings_template_short_term()}</SelectItem>
						<SelectItem value="other" label={m.common_other()}>{m.common_other()}</SelectItem>
						<SelectItem value="receipt" label={m.settings_template_receipt()}>{m.settings_template_receipt()}</SelectItem>
					</SelectContent>
				</Select>
			</label>
			<label class="grid gap-1.5 text-sm font-medium">
				{m.settings_template_locale()}
				<Input bind:value={createDraft.locale} required maxlength={35} dir="ltr" />
			</label>
			<div class="flex justify-end gap-2">
				<Button type="button" variant="outline" onclick={() => (createDialogOpen = false)} disabled={creating}>
					{m.common_cancel()}
				</Button>
				<Button type="submit" disabled={creating || !createDraft.name.trim() || !createDraft.locale.trim()}>
					{creating ? m.common_processing() : m.settings_template_add()}
				</Button>
			</div>
		</form>
	</DialogContent>
</Dialog>
