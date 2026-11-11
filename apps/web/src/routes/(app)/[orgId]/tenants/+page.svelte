<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { Plus } from '@lucide/svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { desktopApi } from '#lib/api/desktop';
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
	const tenantsQuery = createQuery(() => ({
		queryKey: tenantQueryKeys.all(organizationId),
		queryFn: desktopApi.listTenants,
		enabled: !!app.currentUser
	}));
	let open = $state(false);
	let selectedTenant = $state<Tenant | undefined>();
	let saving = $state(false);
	const tenants = $derived(tenantsQuery.data ?? []);

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

{#if app.currentUser}
	<div class="grid gap-4">
		<div class="flex justify-end"><Button class="rounded-md" disabled={app.currentUser.role === 'readonly'} onclick={() => { selectedTenant = undefined; open = true; }}><Plus size={16} />{m.tenants_add()}</Button></div>
		<TenantsDirectory
			{tenants}
			loading={tenantsQuery.isLoading}
			readonly={app.currentUser.role === 'readonly'}
			onEdit={(tenant) => { selectedTenant = tenant; open = true; }}
			onArchive={(tenant) => void archiveTenant(tenant)}
		/>
		<TenantSheet bind:open initial={selectedTenant} busy={saving} onsave={saveTenant} onclose={() => (open = false)} />
	</div>
{/if}
