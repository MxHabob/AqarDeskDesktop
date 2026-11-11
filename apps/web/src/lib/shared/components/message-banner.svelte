<script lang="ts">
		type Props = {
		message: string;
		variant?: 'error' | 'success';
	};

	let { message, variant = 'error' }: Props = $props();
</script>

<div
	class={`rounded-lg border px-4 py-3 text-sm leading-relaxed ${
		variant === 'error'
			? 'border-destructive/25 bg-destructive/10 text-destructive'
			: 'border-primary/40 bg-primary/15 text-foreground'
	}`}
	role={variant === 'error' ? 'alert' : 'status'}
>
	{message}
</div>
