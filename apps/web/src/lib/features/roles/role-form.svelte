<script lang="ts">
	import { untrack } from 'svelte';
	import { Button, Input, SheetFooter } from '@aqardesk/ui';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import {
		permissionDependencies,
		permissionGroups,
		type Permission
	} from '#lib/features/auth/permissions';
	import * as m from '#lib/paraglide/messages';

	export type RoleFormValues = {
		name: string;
		permissions: Permission[];
	};

	type Props = {
		initial?: RolePermissionSet;
		busy: boolean;
		onSubmit: (values: RoleFormValues) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, busy, onSubmit, onCancel }: Props = $props();
	let name = $state(untrack(() => initial?.name ?? ''));
	let selectedPermissions = $state<Permission[]>(
		untrack(
			() =>
				initial?.permissions.filter((permission) => permission !== 'manage_administration') ?? []
		)
	);

	function togglePermission(permission: Permission, checked: boolean) {
		const next = new Set(selectedPermissions);
		if (checked) {
			const addWithDependencies = (item: Permission) => {
				if (next.has(item)) return;
				next.add(item);
				permissionDependencies[item]?.forEach(addWithDependencies);
			};
			addWithDependencies(permission);
		} else {
			next.delete(permission);
			const removeDependents = (prerequisite: Permission) => {
				for (const [dependent, requirements] of Object.entries(permissionDependencies)) {
					if (!requirements.includes(prerequisite) || !next.delete(dependent as Permission)) continue;
					removeDependents(dependent as Permission);
				}
			};
			removeDependents(permission);
		}
		selectedPermissions = [...next];
	}

	function roleTitle(role: RolePermissionSet) {
		switch (role.system_role) {
			case 'admin': return m.settings_permissions_admin();
			case 'staff': return m.settings_permissions_staff();
			case 'readonly': return m.settings_permissions_readonly();
			default: return role.name;
		}
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!initial && !name.trim()) return;
		void onSubmit({ name: name.trim(), permissions: selectedPermissions });
	}
</script>

<form id="role-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto p-4 scrollbar-none">
		{#if initial}
			<div>
				<p class="text-sm font-medium">{m.settings_permissions_role_name()}</p>
				<p class="mt-1 text-sm text-muted-foreground">{roleTitle(initial)}</p>
			</div>
		{:else}
			<div class="grid gap-2">
				<label for="role-name" class="text-sm font-medium">{m.settings_permissions_role_name()}</label>
				<Input
					id="role-name"
					class="rounded-md"
					bind:value={name}
					maxlength={80}
					required
				/>
			</div>
		{/if}

		<fieldset class="grid gap-3">
			<legend class="text-sm font-medium">{m.settings_permissions_role_permissions()}</legend>
			{#each permissionGroups as group (group.label())}
				<div class="grid gap-2">
					<p class="text-xs font-medium text-muted-foreground">{group.label()}</p>
					{#each group.permissions as permission (permission.key)}
						<label class="flex items-center gap-2 text-sm">
							<input
								type="checkbox"
								checked={selectedPermissions.includes(permission.key)}
								disabled={busy}
								class="accent-primary"
								onchange={(event) => togglePermission(permission.key, event.currentTarget.checked)}
							/>
							{permission.label()}
						</label>
					{/each}
				</div>
			{/each}
			<label class="flex items-center gap-2 text-sm text-muted-foreground">
				<input type="checkbox" disabled class="accent-primary" />
				{m.settings_permissions_manage_administration()}
			</label>
			<p class="text-xs text-muted-foreground">{m.settings_permissions_fixed()}</p>
		</fieldset>
	</div>
</form>

<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>
		{m.common_cancel()}
	</Button>
	<Button
		type="submit"
		form="role-form"
		disabled={busy || (!initial && !name.trim())}
	>
		{busy ? m.common_processing() : m.common_save()}
	</Button>
</SheetFooter>
