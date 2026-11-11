<script lang="ts">
	import { goto } from '$app/navigation';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import OverviewPage from '#lib/features/dashboard/OverviewPage.svelte';
	import { getViewPath, type ViewKey } from '#lib/shell/navigation';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);

	function selectView(view: ViewKey) {
		void goto(getViewPath(organizationId, view));
	}

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}
</script>

{#if app.currentUser}
	<OverviewPage
		currentUser={app.currentUser}
		mode={app.mode}
		onSelect={selectView}
		onEditConnection={editConnection}
	/>
{/if}
