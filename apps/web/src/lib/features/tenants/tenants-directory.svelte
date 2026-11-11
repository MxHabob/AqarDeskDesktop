<script lang="ts">
	import { Pencil, Trash2, UsersRound } from '@lucide/svelte';
	import type { Tenant } from '@aqardesk/sdk';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import EmptyState from '#lib/shared/components/empty-state.svelte';
	import * as m from '#lib/paraglide/messages';
	type Props = {
		tenants: Tenant[];
		loading: boolean;
		readonly: boolean;
		onEdit: (tenant: Tenant) => void;
		onArchive: (tenant: Tenant) => void;
	};
	let { tenants, loading, readonly, onEdit, onArchive }: Props = $props();
</script>
<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0"><div class="grid gap-1"><CardTitle class="text-base">{m.nav_tenants()}</CardTitle><CardDescription>{tenants.length} {m.nav_tenants()}</CardDescription></div><span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><UsersRound size={18} /></span></CardHeader>
	{#if loading}<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
	{:else if tenants.length === 0}<EmptyState title={m.tenants_empty()} description={m.dashboard_tenants_description()} />
	{:else}<div class="overflow-x-auto scrollbar-none"><table class="w-full min-w-[38rem] border-collapse text-start text-sm">
		<thead class="border-b border-border text-xs font-medium text-muted-foreground"><tr><th class="px-2 py-3 text-start">{m.tenants_name()}</th><th class="px-2 py-3 text-start">{m.auth_email()}</th><th class="px-2 py-3 text-start">{m.tenants_phone()}</th><th class="px-2 py-3 text-start">{m.users_action()}</th></tr></thead>
		<tbody>{#each tenants as tenant (tenant.id)}<tr class="border-b border-border/60 last:border-0"><td class="px-2 py-3 font-medium">{tenant.full_name}</td><td class="px-2 py-3 text-muted-foreground" dir="ltr">{tenant.email ?? ''}</td><td class="px-2 py-3 text-muted-foreground" dir="ltr">{tenant.phone ?? ''}</td><td class="px-2 py-3"><div class="flex gap-2"><Button variant="outline" size="icon" class="size-8" disabled={readonly} aria-label={m.tenants_edit()} onclick={() => onEdit(tenant)}><Pencil size={15} /></Button><Button variant="outline" size="icon" class="size-8 text-destructive" disabled={readonly} aria-label={m.tenants_archive()} onclick={() => onArchive(tenant)}><Trash2 size={15} /></Button></div></td></tr>{/each}</tbody>
	</table></div>{/if}
</Card>
