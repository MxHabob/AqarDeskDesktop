<script lang="ts">
	import { Button, Sheet, SheetContent, SheetHeader, SheetTitle } from '@aqardesk/ui';
	import { Plus } from '@lucide/svelte';
	import type { CurrencySettings, PaymentInput, Tenant } from '@aqardesk/sdk';
	import PaymentForm from '#lib/features/payments/payment-form.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		tenants: Tenant[];
		currencies: CurrencySettings[];
		busy: boolean;
		onSubmit: (input: PaymentInput) => Promise<boolean>;
	};

	let { tenants, currencies, busy, onSubmit }: Props = $props();
	let open = $state(false);
</script>

<Button class="shrink-0 rounded-md" disabled={tenants.length === 0 || currencies.length === 0} onclick={() => (open = true)}>
	<Plus size={16} />
	{m.payments_record()}
</Button>

<Sheet bind:open>
	<SheetContent side="end" class="max-w-xl">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{m.payments_record()}</SheetTitle>
		</SheetHeader>
		{#if open}
			<PaymentForm
				{tenants}
				{currencies}
				{busy}
				onSubmit={async (input) => {
					const saved = await onSubmit(input);
					if (saved) open = false;
					return saved;
				}}
				onCancel={() => (open = false)}
			/>
		{/if}
	</SheetContent>
</Sheet>
