<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ConnectionSetup from '#lib/features/connection/ConnectionSetup.svelte';

	const app = getAppContext();

	async function saveAndContinue(event: SubmitEvent) {
		if (await app.saveAndConnect(event)) {
			await goto(app.currentUser ? `/${app.currentUser.organization_id}` : '/login');
		}
	}

	async function toggleLanguage() {
		await app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar');
	}
</script>

<ConnectionSetup
	mode={app.mode}
	databasePath={app.databasePath}
	serverUrl={app.serverUrl}
	host={app.host}
	port={app.port}
	databaseName={app.databaseName}
	username={app.username}
	password={app.password}
	sslMode={app.sslMode}
	configured={app.configured}
	busy={app.busy}
	error={app.error}
	notice={app.notice}
	locale={app.locale}
	onSubmit={saveAndContinue}
	onApiCheck={app.checkApi}
	onModeChange={(value) => (app.mode = value)}
	onLanguageChange={toggleLanguage}
	onDatabasePathChange={(value) => (app.databasePath = value)}
	onServerUrlChange={(value) => (app.serverUrl = value)}
	onHostChange={(value) => (app.host = value)}
	onPortChange={(value) => (app.port = value)}
	onDatabaseNameChange={(value) => (app.databaseName = value)}
	onUsernameChange={(value) => (app.username = value)}
	onPasswordChange={(value) => (app.password = value)}
	onSslModeChange={(value) => (app.sslMode = value)}
/>
