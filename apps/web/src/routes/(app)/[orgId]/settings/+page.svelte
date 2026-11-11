<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import SettingsPage from '#lib/features/settings/SettingsPage.svelte';

	const app = getAppContext();

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

<SettingsPage
	locale={app.locale}
	mode={app.mode}
	connected={app.connected}
	busy={app.busy}
	error=""
	notice=""
	updateResult={app.updateResult}
	updateProgress={app.updateProgress}
	progressPercent={app.progressPercent}
	onLanguageChange={(value) => void app.changeLanguage(value)}
	onEditConnection={editConnection}
	onCheckUpdates={app.checkForUpdates}
	onDownloadUpdate={app.downloadUpdate}
	onInstallUpdate={app.installUpdate}
/>
