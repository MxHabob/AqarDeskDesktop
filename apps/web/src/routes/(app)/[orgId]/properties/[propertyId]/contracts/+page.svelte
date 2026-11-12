<script lang="ts">
	import type { PageProps } from './$types';
	import ContractWorkspace from '#lib/features/contracts/contract-workspace.svelte';

	let { data }: PageProps = $props();
</script>

<ContractWorkspace propertyId={data.propertyId} />
