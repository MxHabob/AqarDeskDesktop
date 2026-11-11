<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ConnectionSettingsCard from '#lib/features/settings/connection-settings-card.svelte';
	import LanguageSettingsCard from '#lib/features/settings/language-settings-card.svelte';
	import UpdateSettingsCard from '#lib/features/settings/update-settings-card.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

<div class="grid items-start gap-4 lg:grid-cols-2">
	<LanguageSettingsCard
		locale={app.locale}
		onChange={(value) => void app.changeLanguage(value)}
	/>
	<ConnectionSettingsCard
		mode={app.mode}
		connected={app.connected}
		onEdit={editConnection}
	/>
	<UpdateSettingsCard
		busy={app.busy}
		result={app.updateResult}
		progress={app.updateProgress}
		progressPercent={app.progressPercent}
		onCheck={app.checkForUpdates}
		onDownload={app.downloadUpdate}
		onInstall={app.installUpdate}
	/>
</div>
