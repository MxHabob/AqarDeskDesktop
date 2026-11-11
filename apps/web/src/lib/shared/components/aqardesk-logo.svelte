<script lang="ts">
	import logoMarkup from '#lib/assets/aqardesk-logo.svg?raw';

	let { size = 20, class: className = '' }: { size?: number; class?: string } = $props();
</script>

<span
	class={`aqardesk-logo inline-flex shrink-0 text-foreground ${className}`}
	style={`width: ${(size * 231) / 323}px; height: ${size}px`}
	aria-hidden="true"
>
	{@html logoMarkup}
</span>

<style>
	:global(.aqardesk-logo svg) {
		display: block;
		width: 100%;
		height: 100%;
	}
</style>
