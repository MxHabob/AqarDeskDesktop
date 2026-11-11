<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	onMount(() => {
		void goto(`/${page.params.orgId}/settings`, { replaceState: true });
	});
</script>
