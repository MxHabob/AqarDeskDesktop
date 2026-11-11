<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ConnectionSettingsCard from '#lib/features/settings/ConnectionSettingsCard.svelte';
	import LanguageSettingsCard from '#lib/features/settings/LanguageSettingsCard.svelte';
	import UpdateSettingsCard from '#lib/features/settings/UpdateSettingsCard.svelte';
	import * as m from '#lib/paraglide/messages';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import PageHeader from '#lib/shared/components/PageHeader.svelte';

	const app = getAppContext();

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

<PageHeader title={m.settings_title()} description={m.settings_description()} eyebrow={m.nav_workspace()} />
{#if app.error}<MessageBanner message={app.error} />{/if}
{#if app.notice}<MessageBanner message={app.notice} variant="success" />{/if}

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
