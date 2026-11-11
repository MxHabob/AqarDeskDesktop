<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { Button } from '@aqardesk/ui';
	import { ArrowUpRight } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
</script>

<section class="grid gap-3">
	<h2 class="text-lg font-semibold">{m.nav_payments()}</h2>
	<p class="text-sm text-muted-foreground">{m.payments_property_scope_note()}</p>
	<Button class="w-fit rounded-md" onclick={() => void goto(`/${data.organizationId}/payments`)}>
		<ArrowUpRight size={16} />
		{m.payments_open_register()}
	</Button>
</section>
