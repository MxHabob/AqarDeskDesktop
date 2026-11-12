<script lang="ts">
	import { goto } from '$app/navigation';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import {
		Button,
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle,
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
	import { downloadText, exportRows, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { queryClient } from '#lib/core/query-client';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatDate } from '#lib/core/i18n';
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
	let exporting = $state(false);
	let creating = $state(false);
	let createDialogOpen = $state(false);
	let searchTerm = $state('');
	let typeFilter = $state('all');
	let statusFilter = $state('all');
	let createDraft = $state({
		name: '',
		description: '',
		lease_type: 'residential',
		locale: 'ar'
	});
	const rtl = $derived(getLocale() === 'ar');
	const filteredTemplates = $derived(templates.filter(matchesTemplateFilters));

	function matchesTemplateFilters(template: ContractTemplate) {
		const query = searchTerm.trim().toLocaleLowerCase();
		const matchesSearch =
			!query ||
			[template.name, template.template_key, template.description ?? '', template.locale]
				.some((value) => value.toLocaleLowerCase().includes(query));
		const matchesType = typeFilter === 'all' || template.lease_type === typeFilter;
		const matchesStatus = statusFilter === 'all' || templateStatus(template) === statusFilter;
		return matchesSearch && matchesType && matchesStatus;
	}

	const exportColumns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'name', header: m.settings_template_name() },
		{ id: 'template_key', header: m.settings_template_key() },
		{ id: 'description', header: m.settings_template_description() },
		{ id: 'lease_type', header: m.settings_template_lease_type(), accessor: (row) => templateTypeLabel(row.lease_type) },
		{ id: 'locale', header: m.settings_template_locale() },
		{
			id: 'status',
			header: m.users_status(),
			accessor: (row) =>
				row.is_active
					? m.settings_template_active()
					: Array.isArray(row.versions) && row.versions[0]?.content_body?.trim()
						? m.settings_template_archived()
						: m.settings_template_draft()
		},
		{
			id: 'version_number',
			header: m.settings_template_version(),
			accessor: (row) => (Array.isArray(row.versions) ? row.versions[0]?.version_number : '') ?? ''
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{ id: 'updated_at', header: m.settings_updated_at(), type: 'datetime' }
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

	function hasTemplateContent(template: ContractTemplate) {
		const latest = template.versions[0];
		return typeof latest?.content_body === 'string' && latest.content_body.trim().length > 0;
	}

	function templateStatus(template: ContractTemplate): 'active' | 'draft' | 'archived' {
		if (template.is_active) return 'active';
		return hasTemplateContent(template) ? 'archived' : 'draft';
	}

	function templateStatusLabel(template: ContractTemplate) {
		switch (templateStatus(template)) {
			case 'active': return m.settings_template_active();
			case 'archived': return m.settings_template_archived();
			default: return m.settings_template_draft();
		}
	}

	function templateStatusClass(template: ContractTemplate) {
		switch (templateStatus(template)) {
			case 'active':
				return 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300';
			case 'archived':
				return 'bg-muted text-muted-foreground';
			default:
				return 'bg-amber-500/10 text-amber-700 dark:text-amber-300';
		}
	}

	function templateTypeLabel(type: unknown) {
		switch (type) {
			case 'residential': return m.properties_type_residential();
			case 'commercial': return m.properties_type_commercial();
			case 'short_term': return m.settings_template_short_term();
			case 'receipt': return m.settings_template_receipt();
			default: return m.common_other();
		}
	}

	function openTemplate(template: ContractTemplate) {
		void goto(
			`/${encodeURIComponent(app.currentUser!.organization_id)}/templates/${encodeURIComponent(template.id)}`
		);
	}

	async function exportTemplates() {
		exporting = true;
		try {
			while (templatesQuery.hasNextPage) {
				await fetchNextPageOrThrow(() => templatesQuery.fetchNextPage());
			}
			const allTemplates = templatesQuery.data?.pages.flatMap((page) => page.items) ?? [];
			const csv = exportRows(
				allTemplates.filter(matchesTemplateFilters),
				exportColumns,
				{ format: 'csv', scope: 'all' }
			);
			const date = new Date().toISOString().slice(0, 10);
			const filename = `aqardesk-templates-${date}.csv`;
			downloadText(filename, `\uFEFF${csv}`, 'text/csv;charset=utf-8');
			toast.success(rtl ? 'بدأ تنزيل الملف' : 'Download started', { description: filename });
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			exporting = false;
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

<div class="flex min-h-0 min-w-0 flex-1 flex-col gap-5 p-4 sm:p-6">
	<header class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
		<div class="shrink-0">
			<h1 class="text-xl font-semibold tracking-tight">{m.settings_template_page_title()}</h1>
		</div>
		<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
			<label class="min-w-0 flex-1 sm:min-w-56">
				<Input
					bind:value={searchTerm}
					placeholder={m.common_search()}
					aria-label={m.common_search()}
				/>
			</label>
			<Select
				type="single"
				value={typeFilter}
				items={[
					{ value: 'all', label: m.settings_template_all_types() },
					{ value: 'residential', label: m.properties_type_residential() },
					{ value: 'commercial', label: m.properties_type_commercial() },
					{ value: 'short_term', label: m.settings_template_short_term() },
					{ value: 'other', label: m.common_other() },
					{ value: 'receipt', label: m.settings_template_receipt() }
				]}
				onValueChange={(value) => {
					if (value) typeFilter = value;
				}}
			>
				<SelectTrigger class="w-full sm:w-40"><SelectValue /></SelectTrigger>
				<SelectContent>
					<SelectItem value="all" label={m.settings_template_all_types()}>{m.settings_template_all_types()}</SelectItem>
					<SelectItem value="residential" label={m.properties_type_residential()}>{m.properties_type_residential()}</SelectItem>
					<SelectItem value="commercial" label={m.properties_type_commercial()}>{m.properties_type_commercial()}</SelectItem>
					<SelectItem value="short_term" label={m.settings_template_short_term()}>{m.settings_template_short_term()}</SelectItem>
					<SelectItem value="other" label={m.common_other()}>{m.common_other()}</SelectItem>
					<SelectItem value="receipt" label={m.settings_template_receipt()}>{m.settings_template_receipt()}</SelectItem>
				</SelectContent>
			</Select>
			<Select
				type="single"
				value={statusFilter}
				items={[
					{ value: 'all', label: m.settings_template_all_statuses() },
					{ value: 'active', label: m.settings_template_active() },
					{ value: 'draft', label: m.settings_template_draft() },
					{ value: 'archived', label: m.settings_template_archived() }
				]}
				onValueChange={(value) => {
					if (value) statusFilter = value;
				}}
			>
				<SelectTrigger class="w-full sm:w-40"><SelectValue /></SelectTrigger>
				<SelectContent>
					<SelectItem value="all" label={m.settings_template_all_statuses()}>{m.settings_template_all_statuses()}</SelectItem>
					<SelectItem value="active" label={m.settings_template_active()}>{m.settings_template_active()}</SelectItem>
					<SelectItem value="draft" label={m.settings_template_draft()}>{m.settings_template_draft()}</SelectItem>
					<SelectItem value="archived" label={m.settings_template_archived()}>{m.settings_template_archived()}</SelectItem>
				</SelectContent>
			</Select>
			<Button variant="outline" onclick={exportTemplates} disabled={exporting || loading}>
				{m.settings_template_export_csv()}
			</Button>
			<Button onclick={createTemplate} disabled={busy}>
				{m.settings_template_add()}
			</Button>
		</div>
	</header>

	<div class="min-h-0 flex-1 overflow-y-auto scrollbar-none">
		{#if loading && templates.length === 0}
			<div class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
				{#each [0, 1, 2] as placeholder (placeholder)}
					<Card class="h-52 animate-pulse p-3">
						<CardContent class="flex h-full flex-col gap-4 p-0">
							<div class="grid gap-2">
								<div class="h-4 w-2/3 rounded bg-muted"></div>
								<div class="h-3 w-1/2 rounded bg-muted"></div>
							</div>
							<div class="h-3 w-full rounded bg-muted"></div>
							<div class="h-3 w-3/4 rounded bg-muted"></div>
							<div class="mt-auto h-8 rounded bg-muted"></div>
						</CardContent>
					</Card>
				{/each}
			</div>
		{:else if filteredTemplates.length > 0}
			<div class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
				{#each filteredTemplates as template (template.id)}
					<Card class="group flex min-h-[210px] min-w-0 flex-col justify-between gap-4 p-3 transition-colors hover:border-primary/50">
						<CardHeader class="gap-3 p-0">
							<div class="flex items-start justify-between gap-3">
								<div class="min-w-0">
									<CardTitle class="truncate text-base">{template.name}</CardTitle>
									<p dir="ltr" class="mt-1 truncate text-xs text-muted-foreground">{template.template_key}</p>
								</div>
								<span class={`shrink-0 rounded-full px-2.5 py-1 text-xs font-medium ${templateStatusClass(template)}`}>
									{templateStatusLabel(template)}
								</span>
							</div>
							<CardDescription class="line-clamp-2 min-h-10">
								{template.description || m.settings_template_description()}
							</CardDescription>
						</CardHeader>
						<CardContent class="flex flex-wrap gap-2 p-0">
							<span class="rounded-md bg-muted px-2.5 py-1 text-xs text-muted-foreground">
								{templateTypeLabel(template.lease_type)}
							</span>
							<span dir="ltr" class="rounded-md bg-muted px-2.5 py-1 text-xs text-muted-foreground">
								{template.locale.toUpperCase()}
							</span>
							{#if template.versions[0]}
								<span class="rounded-md bg-muted px-2.5 py-1 text-xs text-muted-foreground">
									{m.settings_template_version()} {template.versions[0].version_number}
								</span>
							{/if}
						</CardContent>
						<CardFooter class="mt-auto flex flex-wrap items-center justify-between gap-2 border-t border-border/60 p-0 pt-3">
							<p class="text-xs text-muted-foreground">
								{m.settings_updated_at()}: {formatDate(template.updated_at, 'date', undefined, app.organizationTimezone)}
							</p>
							<div class="flex items-center gap-2">
								<Button variant="outline" size="sm" onclick={() => openTemplate(template)} disabled={busy}>
									{m.settings_template_edit()}
								</Button>
								{#if template.is_active}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => void changeActive(template)}
										disabled={busy}
									>
										{m.settings_template_disable()}
									</Button>
								{:else}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => void changeActive(template)}
										disabled={busy || !hasTemplateContent(template)}
										title={!hasTemplateContent(template)
											? m.settings_template_enable_content_first()
											: undefined}
									>
										{m.settings_template_enable()}
									</Button>
								{/if}
							</div>
						</CardFooter>
					</Card>
				{/each}
			</div>
		{:else}
			<div class="flex min-h-60 flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/70 px-6 text-center">
				<p class="max-w-md text-sm text-muted-foreground">
					{searchTerm || typeFilter !== 'all' || statusFilter !== 'all'
						? m.settings_template_no_filter_results()
						: m.settings_template_empty()}
				</p>
			</div>
		{/if}

		{#if templatesQuery.hasNextPage}
			<div class="flex justify-center py-5">
				<Button
					variant="outline"
					onclick={() => fetchNextPageOrThrow(() => templatesQuery.fetchNextPage())}
					disabled={templatesQuery.isFetchingNextPage}
				>
					{templatesQuery.isFetchingNextPage ? m.common_loading() : m.settings_load_more()}
				</Button>
			</div>
		{/if}
	</div>
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
