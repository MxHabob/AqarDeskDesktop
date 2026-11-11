<script lang="ts">
	import { onMount } from 'svelte';
	import * as m from '#lib/paraglide/messages';

	let {
		value,
		persistId,
		onChange
	}: {
		value: number;
		persistId: string;
		onChange: (value: number) => void;
	} = $props();

	let pointerStart = 0;
	let initialValue = 0;
	let activePointer: number | null = null;
	let handle: HTMLButtonElement;

	onMount(() => {
		const saved = Number(localStorage.getItem(`resizer:${persistId}`));
		if (Number.isInteger(saved) && saved >= 0 && saved <= 3) onChange(saved);
	});

	function onPointerDown(event: PointerEvent) {
		event.preventDefault();
		pointerStart = event.clientX;
		initialValue = value;
		activePointer = event.pointerId;
		handle.setPointerCapture(event.pointerId);
	}

	function onPointerMove(event: PointerEvent) {
		if (activePointer !== event.pointerId) return;
		const isRtl = getComputedStyle(handle).direction === 'rtl';
		const delta = (event.clientX - pointerStart) * (isRtl ? -1 : 1);
		onChange(Math.max(0, Math.min(3, initialValue + Math.round(delta / 72))));
	}

	function onPointerUp(event: PointerEvent) {
		if (activePointer !== event.pointerId) return;
		activePointer = null;
		localStorage.setItem(`resizer:${persistId}`, String(value));
		if (handle.hasPointerCapture(event.pointerId)) handle.releasePointerCapture(event.pointerId);
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		event.preventDefault();
		const direction = event.key === 'ArrowRight' ? 1 : -1;
		const next = Math.max(0, Math.min(3, value + direction));
		onChange(next);
		localStorage.setItem(`resizer:${persistId}`, String(next));
	}

	function resetWidth() {
		onChange(2);
		localStorage.setItem(`resizer:${persistId}`, '2');
	}
</script>

<button
	bind:this={handle}
	type="button"
	role="slider"
	aria-orientation="vertical"
	aria-valuemin={0}
	aria-valuemax={3}
	aria-valuenow={value}
	aria-label={m.sidebar_resize()}
	title={m.sidebar_resize()}
	class="group relative z-10 -mx-1 w-2 shrink-0 cursor-col-resize touch-none bg-transparent focus-visible:outline-none"
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	ondblclick={resetWidth}
	onkeydown={onKeydown}
>
	<span class="absolute inset-y-2 start-1/2 w-0.5 -translate-x-1/2 rounded-full bg-transparent transition-colors group-hover:bg-primary/40 group-focus-visible:bg-primary/60"></span>
</button>
