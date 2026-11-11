<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ConnectionSettingsCard from '#lib/features/settings/connection-settings-card.svelte';
	import LanguageSettingsCard from '#lib/features/settings/language-settings-card.svelte';
	import UpdateSettingsCard from '#lib/features/settings/update-settings-card.svelte';
	import SettingsBottomBar from '#lib/shared/components/settings-bottom-bar.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let draftLocale = $state(app.locale);
	let savingLocale = $state(false);
	const localeDirty = $derived(draftLocale !== app.locale);

	async function saveLocale() {
		savingLocale = true;
		try {
			await app.changeLanguage(draftLocale);
		} finally {
			savingLocale = false;
		}
	}

	function resetLocale() {
		draftLocale = app.locale;
	}

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

<form
	class="grid items-start gap-4 pb-20 lg:grid-cols-2"
	onsubmit={(event) => {
		event.preventDefault();
		void saveLocale();
	}}
>
	<LanguageSettingsCard locale={draftLocale} onChange={(value) => (draftLocale = value)} />
	<ConnectionSettingsCard mode={app.mode} connected={app.connected} onEdit={editConnection} />
	<UpdateSettingsCard
		busy={app.busy}
		result={app.updateResult}
		progress={app.updateProgress}
		progressPercent={app.progressPercent}
		onCheck={app.checkForUpdates}
		onDownload={app.downloadUpdate}
		onInstall={app.installUpdate}
	/>
	<SettingsBottomBar
		visible={localeDirty}
		isSaving={savingLocale}
		onsave={saveLocale}
		onreset={resetLocale}
	/>
</form>
