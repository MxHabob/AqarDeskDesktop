<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';

	let { params }: PageProps = $props();
	onMount(() => {
		void goto(
			`/${encodeURIComponent(params.orgId)}/templates/${encodeURIComponent(params.templateId)}`,
			{ replaceState: true }
		);
	});
</script>
