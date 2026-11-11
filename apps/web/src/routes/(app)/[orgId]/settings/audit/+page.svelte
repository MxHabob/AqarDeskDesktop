<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { AuditEntry } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	const pageSize = 50;
	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let hasMore = $state(false);

	onMount(() => {
		void load(false);
	});

	async function load(more: boolean) {
		if (more) loadingMore = true;
		else loading = true;
		try {
			const next = await desktopApi.listAuditEvents(pageSize, more ? entries.length : 0);
			entries = more ? [...entries, ...next] : next;
			hasMore = next.length === pageSize;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
			loadingMore = false;
		}
	}

	function formatTimestamp(value: string) {
		const timestamp = new Date(value);
		return Number.isNaN(timestamp.getTime()) ? value : timestamp.toLocaleString();
	}
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_audit()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_audit_description()}</p>
</header>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if entries.length === 0}
	<p class="py-8 text-sm text-muted-foreground">{m.settings_audit_empty()}</p>
{:else}
	<div class="grid divide-y divide-border/70 border-y border-border/70">
		{#each entries as entry (entry.id)}
			<article class="grid gap-2 py-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
				<div class="grid min-w-0 gap-1">
					<p class="text-sm font-medium">
						{entry.actor_name ?? entry.actor_user_id ?? m.settings_audit_actor()}
						<span class="font-normal text-muted-foreground">· {entry.action}</span>
					</p>
					<p class="truncate text-xs text-muted-foreground" dir="ltr">
						{entry.entity_type} / {entry.entity_id}
					</p>
					{#if entry.details_json}
						<details class="text-xs text-muted-foreground">
							<summary class="w-fit cursor-pointer">{m.settings_audit_details()}</summary>
							<pre class="mt-2 overflow-x-auto whitespace-pre-wrap rounded-md bg-muted/50 p-2" dir="ltr">{entry.details_json}</pre>
						</details>
					{/if}
				</div>
				<time class="text-xs text-muted-foreground" datetime={entry.created_at}>
					{formatTimestamp(entry.created_at)}
				</time>
			</article>
		{/each}
	</div>
	{#if hasMore}
		<div class="flex justify-center">
			<Button variant="outline" disabled={loadingMore} onclick={() => void load(true)}>
				{loadingMore ? m.common_processing() : m.settings_load_more()}
			</Button>
		</div>
	{/if}
{/if}
