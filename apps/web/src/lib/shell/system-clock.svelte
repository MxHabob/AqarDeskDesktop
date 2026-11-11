<script lang="ts">
	import { onMount } from 'svelte';
	import { Popover } from 'bits-ui';
	import { Clock } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import { formatDate } from '#lib/core/i18n';
	import { getHtmlDirection, getInlineEndSide } from '#lib/core/i18n/direction';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let now = $state(new Date());
	let open = $state(false);
	const timeZone = $derived(app.organizationTimezone);
	const timeLabel = $derived(formatDate(now, 'time', undefined, timeZone));
	const dateLabel = $derived(formatDate(now, 'date', { dateStyle: 'full' }, timeZone));
	const direction = $derived(getHtmlDirection());
	const popoverSide = $derived(getInlineEndSide(direction));

	onMount(() => {
		const updateTime = () => {
			now = new Date();
		};
		const timer = window.setInterval(updateTime, 15_000);
		window.addEventListener('focus', updateTime);

		return () => {
			window.clearInterval(timer);
			window.removeEventListener('focus', updateTime);
		};
	});
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		<Button
			variant="outline"
			size="icon"
			class="size-6 rounded-md"
			aria-label={open ? m.sidebar_clock_close() : m.sidebar_clock_open()}
			title={timeLabel}
			aria-expanded={open}
		>
			<Clock size={14} />
		</Button>
	</Popover.Trigger>
	<Popover.Content
		side={popoverSide}
		align="end"
		sideOffset={10}
		dir={direction}
		class="z-[var(--layer-popover)] w-[min(280px,calc(100vw-4rem))] rounded-xl bg-transparent p-4 text-start text-popover-foreground shadow-floating backdrop-blur-md"
	>
		<div class="grid justify-items-center gap-2">
			<time
				datetime={now.toISOString()}
				class="text-4xl font-semibold tracking-tight tabular-nums text-foreground"
			>
				{timeLabel}
			</time>
			<span class="text-center text-sm font-medium text-muted-foreground">{dateLabel}</span>
			<span class="text-xs text-muted-foreground">{timeZone}</span>
		</div>
	</Popover.Content>
</Popover.Root>
