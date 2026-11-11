<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import {
		Button,
		Input,
		Sheet,
		SheetContent,
		SheetDescription,
		SheetFooter,
		SheetHeader,
		SheetTitle
	} from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		busy: boolean;
		name: string;
		email: string;
		password: string;
		role: string;
		onAdd: (event: SubmitEvent) => Promise<boolean>;
		onCancel: () => void;
		onNameChange: (value: string) => void;
		onEmailChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onRoleChange: (value: string) => void;
	};

	let {
		busy,
		name,
		email,
		password,
		role,
		onAdd,
		onCancel,
		onNameChange,
		onEmailChange,
		onPasswordChange,
		onRoleChange
	}: Props = $props();

	let open = $state(false);
	let wasOpen = false;

	$effect(() => {
		if (open) {
			wasOpen = true;
		} else if (wasOpen) {
			wasOpen = false;
			onCancel();
		}
	});

	async function addUser(event: SubmitEvent) {
		if (await onAdd(event)) open = false;
	}
</script>

<Button class="shrink-0 rounded-md" onclick={() => (open = true)}>
	<Plus size={16} />{m.users_add()}
</Button>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="border-b border-border px-5 py-5">
			<SheetTitle>{m.users_add_title()}</SheetTitle>
			<SheetDescription>{m.users_description()}</SheetDescription>
		</SheetHeader>
		<form class="grid min-h-0 flex-1 gap-4 overflow-y-auto p-5 scrollbar-none" onsubmit={addUser}>
			<FormField label={m.users_name()} id="new-user-name">
				<Input
					id="new-user-name"
					value={name}
					oninput={(event) => onNameChange(event.currentTarget.value)}
					required
					autocomplete="name"
					class="rounded-md"
				/>
			</FormField>
			<FormField label={m.auth_email()} id="new-user-email">
				<Input
					id="new-user-email"
					type="email"
					value={email}
					oninput={(event) => onEmailChange(event.currentTarget.value)}
					required
					dir="ltr"
					autocomplete="email"
					class="rounded-md"
				/>
			</FormField>
			<FormField
				label={m.auth_password()}
				id="new-user-password"
				description={m.auth_password_hint()}
			>
				<Input
					id="new-user-password"
					type="password"
					value={password}
					oninput={(event) => onPasswordChange(event.currentTarget.value)}
					required
					minlength={12}
					dir="ltr"
					autocomplete="new-password"
					class="rounded-md"
				/>
			</FormField>
			<FormField label={m.users_role()} id="new-user-role">
				<select
					id="new-user-role"
					value={role}
					onchange={(event) => onRoleChange(event.currentTarget.value)}
					class="h-9 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground shadow-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
				>
					<option value="staff">{m.users_role_staff()}</option>
					<option value="admin">{m.users_role_admin()}</option>
				</select>
			</FormField>
			<SheetFooter class="border-t border-border pt-4">
				<Button
					type="button"
					variant="outline"
					class="rounded-md"
					disabled={busy}
					onclick={() => (open = false)}
				>
					{m.common_cancel()}
				</Button>
				<Button type="submit" class="rounded-md" disabled={busy}>
					{busy ? m.common_processing() : m.users_add()}
				</Button>
			</SheetFooter>
		</form>
	</SheetContent>
</Sheet>
