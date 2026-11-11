<script lang="ts">
		type Props = {
		message: string;
		variant?: 'error' | 'success';
	};

	let { message, variant = 'error' }: Props = $props();
</script>

<div
	class="rounded-md border px-3 py-2.5 text-sm leading-relaxed"
	class:border-rose-200={variant === 'error'}
	class:bg-rose-50={variant === 'error'}
	class:text-rose-800={variant === 'error'}
	class:border-emerald-200={variant === 'success'}
	class:bg-emerald-50={variant === 'success'}
	class:text-emerald-800={variant === 'success'}
	role={variant === 'error' ? 'alert' : 'status'}
>
	{message}
</div>
