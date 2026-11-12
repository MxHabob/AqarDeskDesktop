<script lang="ts">
	import { Button } from '@aqardesk/ui';
	import { Plus } from '@lucide/svelte';
	import type { ContractDocument } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import { AqarDeskGrid, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		contracts: ContractDocument[];
		loading: boolean;
		emptyDescription: string;
		canAdd: boolean;
		onAdd: () => void;
		hasMore?: boolean;
		onloadmore?: () => Promise<unknown>;
	};

	let {
		contracts,
		loading,
		emptyDescription,
		canAdd,
		onAdd,
		hasMore = false,
		onloadmore
	}: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	const tenants = $derived.by(() => {
		const byTenant = new Map<
			string,
			{
				id: string;
				tenant_id: string;
				tenant_name: string;
				contract_count: number;
				last_contract_at: string;
			}
		>();

		for (const contract of contracts) {
			const tenant = byTenant.get(contract.tenant_id);
			if (tenant) {
				tenant.contract_count += 1;
				if (contract.created_at > tenant.last_contract_at) {
					tenant.last_contract_at = contract.created_at;
				}
			} else {
				byTenant.set(contract.tenant_id, {
					id: contract.tenant_id,
					tenant_id: contract.tenant_id,
					tenant_name: contract.tenant_name,
					contract_count: 1,
					last_contract_at: contract.created_at
				});
			}
		}

		return Array.from(byTenant.values());
	});
	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'tenant_name', header: m.tenants_name(), width: 220 },
		{ id: 'tenant_id', header: m.payments_tenant_id(), width: 220 },
		{
			id: 'contract_count',
			header: m.contracts_saved_title(),
			type: 'integer',
			width: 160
		},
		{
			id: 'last_contract_at',
			header: m.settings_created_at(),
			type: 'datetime',
			width: 190
		}
	]);
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={tenants}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		{hasMore}
		{onloadmore}
		emptyTitle={m.tenants_empty()}
		{emptyDescription}
	>
		{#snippet toolbarActions()}
			<Button class="rounded-md" disabled={!canAdd} onclick={onAdd}>
				<Plus size={16} />{m.tenants_add()}
			</Button>
		{/snippet}
	</AqarDeskGrid>
</div>
