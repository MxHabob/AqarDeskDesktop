<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { desktopApi } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { queryClient } from '#lib/core/query-client';
	import TenantSheet from '#lib/features/tenants/tenant-sheet.svelte';
	import TenantsDirectory from '#lib/features/tenants/tenants-directory.svelte';
	import { tenantQueryKeys } from '#lib/features/tenants/queries';
	import type { Tenant, TenantInput } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	const canView = $derived(
		hasPermission(app.currentUser?.role, 'view_tenants', app.currentUser?.permissions)
	);
	const canCreate = $derived(
		hasPermission(app.currentUser?.role, 'create_tenants', app.currentUser?.permissions)
	);
	const canUpdate = $derived(
		hasPermission(app.currentUser?.role, 'update_tenants', app.currentUser?.permissions)
	);
	const canArchive = $derived(
		hasPermission(app.currentUser?.role, 'archive_tenants', app.currentUser?.permissions)
	);
	const pageSize = 50;
	const tenantsQuery = createInfiniteQuery(() => ({
		queryKey: tenantQueryKeys.pages(organizationId),
		queryFn: ({ pageParam }) => desktopApi.listTenantsPage(pageSize, pageParam),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser && canView
	}));
	let open = $state(false);
	let selectedTenant = $state<Tenant | undefined>();
	let saving = $state(false);
	const tenants = $derived(tenantsQuery.data?.pages.flatMap((page) => page.items) ?? []);

	async function saveTenant(values: TenantInput) {
		saving = true;
		try {
			if (selectedTenant) await desktopApi.updateTenant(selectedTenant.id, values);
			else await desktopApi.createTenant(values);
			await queryClient.invalidateQueries({ queryKey: tenantQueryKeys.all(organizationId) });
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			saving = false;
		}
	}

	async function archiveTenant(tenant: Tenant) {
		const confirmed = await confirmStore.prompt({
			title: m.tenants_archive_confirm_title(),
			description: m.tenants_archive_confirm_description({ name: tenant.full_name }),
			confirmText: m.tenants_archive(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;
		try {
			await desktopApi.archiveTenant(tenant.id);
			await queryClient.invalidateQueries({ queryKey: tenantQueryKeys.all(organizationId) });
			toast.success(m.tenants_archived());
		} catch (cause) {
			app.setError(cause);
		}
	}

	$effect(() => {
		if (tenantsQuery.error) app.setError(tenantsQuery.error);
	});
</script>

{#if app.currentUser && canView}
	<div class="flex min-h-0 flex-1 flex-col">
		<TenantsDirectory
			{tenants}
			loading={tenantsQuery.isLoading}
			hasMore={tenantsQuery.hasNextPage}
			onloadmore={() => fetchNextPageOrThrow(() => tenantsQuery.fetchNextPage())}
			{canCreate}
			{canUpdate}
			{canArchive}
			onAdd={() => { selectedTenant = undefined; open = true; }}
			onEdit={(tenant) => { selectedTenant = tenant; open = true; }}
			onArchive={(tenant) => void archiveTenant(tenant)}
		/>
		<TenantSheet bind:open initial={selectedTenant} busy={saving} onsave={saveTenant} onclose={() => (open = false)} />
	</div>
{/if}
