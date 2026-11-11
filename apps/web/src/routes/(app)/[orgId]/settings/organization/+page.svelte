<script lang="ts">
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';
	import SettingsCapabilityNotice from '#lib/features/settings/settings-capability-notice.svelte';

	const app = getAppContext();
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_organization()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_organization_description()}</p>
</header>
<SettingsCapabilityNotice
	title={m.settings_organization()}
	description={m.settings_organization_unavailable({ id: app.currentUser?.organization_id ?? '' })}
/>
