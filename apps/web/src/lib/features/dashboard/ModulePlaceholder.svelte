<script lang="ts">
	import { ArrowUpRight, UsersRound } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from '#lib/shell/navigation';
	import EmptyState from '#lib/shared/components/EmptyState.svelte';

	type Props = {
		title: string;
		description: string;
		currentUser: User;
		onSelect: (view: ViewKey) => void;
	};

	let { title, description, currentUser, onSelect }: Props = $props();
</script>

<EmptyState title={m.module_empty_title()} description={m.module_empty_description({ section: title.toLocaleLowerCase() })}>
	<div class="flex flex-wrap justify-center gap-2">
		<Button variant="outline" class="rounded-md" onclick={() => onSelect('overview')}><ArrowUpRight size={15} />{m.nav_overview()}</Button>
		{#if currentUser.role === 'admin'}
			<Button variant="ghost" class="rounded-md text-foreground" onclick={() => onSelect('users')}><UsersRound size={15} />{m.nav_users()}</Button>
		{/if}
	</div>
</EmptyState>
