<script lang="ts">
	import { Check, CircleAlert, Info, X } from '@lucide/svelte';
	import { fade, fly } from 'svelte/transition';
	import { toastManager } from './toast.svelte.js';

	const visibleToasts = $derived(toastManager.items);
</script>

<div
	class="pointer-events-none fixed left-1/2 top-3 z-[var(--layer-toast)] flex w-[min(24rem,calc(100vw-1.5rem))] -translate-x-1/2 flex-col gap-2 sm:top-4"
	role="region"
	aria-label="Notifications"
>
	{#each visibleToasts as item (item.id)}
		{#if item.open}
			<div
				class="pointer-events-auto flex items-start gap-3 border border-border bg-card p-3 text-card-foreground shadow-floating"
				data-variant={item.variant}
				role="status"
				aria-live={item.variant === 'error' ? 'assertive' : 'polite'}
				in:fly={{ y: -8, duration: 140 }}
				out:fade={{ duration: 140 }}
			>
				<span
					class={`mt-0.5 shrink-0 ${
						item.variant === 'success'
							? 'text-primary'
							: item.variant === 'error' || item.variant === 'warning'
								? 'text-destructive'
								: 'text-muted-foreground'
					}`}
				>
					{#if item.variant === 'success'}
						<Check size={16} />
					{:else if item.variant === 'error' || item.variant === 'warning'}
						<CircleAlert size={16} />
					{:else}
						<Info size={16} />
					{/if}
				</span>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-medium leading-5">{item.title}</p>
					{#if item.description}
						<p class="mt-1 text-xs leading-5 text-muted-foreground">{item.description}</p>
					{/if}
				</div>
				<button
					type="button"
					class="shrink-0 p-0.5 text-muted-foreground transition-colors hover:text-foreground"
					aria-label="Dismiss notification"
					onclick={() => toastManager.dismiss(item.id)}
				>
					<X size={14} />
				</button>
			</div>
		{/if}
	{/each}
</div>
