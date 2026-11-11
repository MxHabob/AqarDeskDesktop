<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import AuthPanel from '#lib/features/auth/AuthPanel.svelte';

	const app = getAppContext();

	async function authenticate(event: SubmitEvent) {
		if (app.needsAdmin) await app.createInitialAdmin(event);
		else await app.signIn(event);
		if (app.currentUser) await goto(`/${app.currentUser.organization_id}`);
	}

	async function openConnectionSettings() {
		app.clearMessages();
		await goto('/connection');
	}
</script>

<AuthPanel
	needsAdmin={app.needsAdmin}
	busy={app.busy}
	error={app.error}
	notice={app.notice}
	locale={app.locale}
	organizationName={app.organizationName}
	name={app.fullName}
	email={app.email}
	password={app.accountPassword}
	onSubmit={authenticate}
	onOpenConnection={openConnectionSettings}
	onLanguageChange={() => void app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar')}
	onOrganizationNameChange={(value) => (app.organizationName = value)}
	onNameChange={(value) => (app.fullName = value)}
	onEmailChange={(value) => (app.email = value)}
	onPasswordChange={(value) => (app.accountPassword = value)}
/>
