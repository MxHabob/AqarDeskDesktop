<script lang="ts">
	import { Button, Input } from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		needsAdmin: boolean;
		busy: boolean;
		organizationName: string;
		name: string;
		email: string;
		password: string;
		onSubmit: (event: SubmitEvent) => void;
		onOrganizationNameChange: (value: string) => void;
		onNameChange: (value: string) => void;
		onEmailChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
	};

	let {
		needsAdmin,
		busy,
		organizationName,
		name,
		email,
		password,
		onSubmit,
		onOrganizationNameChange,
		onNameChange,
		onEmailChange,
		onPasswordChange
	}: Props = $props();
</script>

<form class="grid gap-4" onsubmit={onSubmit}>
	{#if needsAdmin}
		<FormField label={m.auth_organization()} id="organization-name">
			<Input
				id="organization-name"
				value={organizationName}
				oninput={(event) => onOrganizationNameChange(event.currentTarget.value)}
				required
				autocomplete="organization"
				class="h-10 rounded-md"
			/>
		</FormField>
		<FormField label={m.auth_full_name()} id="full-name">
			<Input
				id="full-name"
				value={name}
				oninput={(event) => onNameChange(event.currentTarget.value)}
				required
				autocomplete="name"
				class="h-10 rounded-md"
			/>
		</FormField>
	{/if}

	<FormField label={m.auth_email()} id="account-email">
		<Input
			id="account-email"
			type="email"
			value={email}
			oninput={(event) => onEmailChange(event.currentTarget.value)}
			required
			dir="ltr"
			autocomplete="email"
			class="h-10 rounded-md"
		/>
	</FormField>
	<FormField label={m.auth_password()} id="account-password" description={needsAdmin ? m.auth_password_hint() : undefined}>
		<Input
			id="account-password"
			type="password"
			value={password}
			oninput={(event) => onPasswordChange(event.currentTarget.value)}
			required
			minlength={needsAdmin ? 12 : undefined}
			dir="ltr"
			autocomplete={needsAdmin ? 'new-password' : 'current-password'}
			class="h-10 rounded-md"
		/>
	</FormField>
	<Button class="mt-1 h-10 w-full rounded-md" type="submit" disabled={busy}>
		{busy ? m.common_processing() : needsAdmin ? m.auth_create_admin() : m.auth_sign_in()}
	</Button>
</form>
