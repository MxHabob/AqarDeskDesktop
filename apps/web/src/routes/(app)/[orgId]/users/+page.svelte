<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import * as m from '#lib/paraglide/messages';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	onMount(() => {
		void goto(`/${data.orgId}/settings/users`, { replaceState: true });
	});
</script>
<p class="text-sm text-muted-foreground">{m.common_loading()}</p>
