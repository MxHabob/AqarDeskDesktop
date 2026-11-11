<script lang="ts">
	import {
		Button,
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		Input,
		Label,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		toast
	} from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { openExternalUrl } from '#lib/core/external-url';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	type AppInfo = {
		name: string;
		version?: string;
	};

	let info = $state<AppInfo | null>(null);
	let subject = $state('');
	let category = $state('technical_support');
	let email = $state('');
	let message = $state('');
	let isSubmitting = $state(false);
	let isComposeOpened = $state(false);
	let isCopied = $state(false);
	let submitError = $state<string | null>(null);
	let composeUrl = $state<string | null>(null);
	let showDiagnosticsPreview = $state(false);
	let isLoadingInfo = $state(false);
	let infoError = $state<string | null>(null);
	const categories = $derived([
		{ value: 'technical_support', label: m.system_support_category_technical() },
		{ value: 'onboarding_setup', label: m.system_support_category_onboarding() },
		{ value: 'property_management', label: m.system_support_category_property_management() },
		{ value: 'feature_request', label: m.system_support_category_feature_request() },
		{ value: 'other', label: m.system_support_category_general() }
	]);
	const selectedCategory = $derived(categories.find((item) => item.value === category));
	const diagnosticPayload = $derived({
		application: info?.name,
		version: info?.version,
		build_profile: import.meta.env.MODE,
		platform: typeof navigator !== 'undefined' ? navigator.platform : undefined,
		locale: typeof navigator !== 'undefined' ? navigator.language : undefined,
		timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
		user_agent: typeof navigator !== 'undefined' ? navigator.userAgent : undefined
	});

	$effect(() => {
		if (open && !info && !isLoadingInfo) void loadAppInfo();
	});

	async function loadAppInfo() {
		isLoadingInfo = true;
		infoError = null;
		info = { name: m.app_name() };
		try {
			if ('__TAURI_INTERNALS__' in window) {
				const { getVersion } = await import('@tauri-apps/api/app');
				info = { name: m.app_name(), version: await getVersion() };
			} else {
				info = { name: m.app_name(), version: 'web' };
			}
		} catch (cause) {
			console.error('Failed to load application version for support diagnostics.', cause);
			infoError = cause instanceof Error ? cause.message : String(cause);
		} finally {
			isLoadingInfo = false;
		}
	}

	function createSupportReport() {
		return `${m.system_support_report_title()}
${m.system_support_category()}: ${selectedCategory?.label ?? category}
${m.system_support_subject()}: ${subject.trim()}
${m.system_support_contact_email()}: ${email.trim() || '-'}

${m.system_support_details()}:
${message.trim()}

${m.system_support_diagnostics()}:
${JSON.stringify({ ...diagnosticPayload, timestamp: new Date().toISOString() }, null, 2)}`;
	}

	async function copyDiagnosticReport() {
		submitError = null;
		try {
			await navigator.clipboard.writeText(createSupportReport());
			isCopied = true;
			window.setTimeout(() => (isCopied = false), 2500);
		} catch (cause) {
			console.error('Failed to copy support diagnostics.', cause);
			submitError = cause instanceof Error ? cause.message : String(cause);
			toast.error(submitError);
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (!subject.trim() || !message.trim()) return;

		isSubmitting = true;
		submitError = null;
		isComposeOpened = false;
		try {
			const mailto = new URL('mailto:support@mulverse.com');
			mailto.searchParams.set('subject', `[AqarDesk] ${subject.trim()}`);
			mailto.searchParams.set('body', createSupportReport());
			composeUrl = mailto.toString();
			await openExternalUrl(composeUrl);
			isComposeOpened = true;
		} catch (cause) {
			console.error('Failed to open the support email composer.', cause);
			submitError = cause instanceof Error ? cause.message : String(cause);
			toast.error(submitError);
		} finally {
			isSubmitting = false;
		}
	}

	function resetForm() {
		subject = '';
		category = 'technical_support';
		email = '';
		message = '';
		isComposeOpened = false;
		composeUrl = null;
		submitError = null;
		open = false;
	}
</script>

<Dialog bind:open>
	<DialogContent
		class="flex max-h-[90dvh] max-w-2xl flex-col overflow-hidden"
	>
		<DialogHeader>
			<DialogTitle>{m.system_support_diagnostics_title()}</DialogTitle>
		</DialogHeader>

		{#if isComposeOpened}
			<div class="my-auto space-y-4 py-6 text-center">
				<h3 class="font-semibold">{m.system_support_compose_ready()}</h3>
				<p class="text-sm text-muted-foreground">{m.system_support_compose_instructions()}</p>
				<div class="flex items-center justify-center gap-3">
					<Button variant="outline" size="sm" onclick={() => void copyDiagnosticReport()}>
						{isCopied ? m.system_support_copied() : m.system_support_copy_full_report()}
					</Button>
					<Button size="sm" onclick={resetForm}>{m.system_support_done()}</Button>
				</div>
			</div>
		{:else}
			<form
				id="support-form"
				class="flex-1 space-y-4 overflow-y-auto p-6"
				onsubmit={(event) => void handleSubmit(event)}
			>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="space-y-1.5">
						<Label for="support-category">{m.system_support_category()}</Label>
						<Select
							type="single"
							value={category}
							onValueChange={(value) => (category = value ?? 'technical_support')}
						>
							<SelectTrigger id="support-category" class="h-9 w-full text-xs">
								{selectedCategory?.label ?? category}
							</SelectTrigger>
							<SelectContent>
								{#each categories as item (item.value)}
									<SelectItem value={item.value}>{item.label}</SelectItem>
								{/each}
							</SelectContent>
						</Select>
					</div>
					<div class="space-y-1.5">
						<Label for="support-email">{m.system_support_contact_email()}</Label>
						<Input
							id="support-email"
							type="email"
							dir="ltr"
							placeholder="name@example.com"
							bind:value={email}
							class="h-9 text-xs"
						/>
					</div>
				</div>

				<div class="space-y-1.5">
					<Label for="support-subject">{m.system_support_subject()}</Label>
					<Input
						id="support-subject"
						required
						maxlength={160}
						placeholder={m.system_support_subject_placeholder()}
						bind:value={subject}
						class="h-9 text-xs"
					/>
				</div>

				<div class="space-y-1.5">
					<Label for="support-message">{m.system_support_details()}</Label>
					<textarea
						id="support-message"
						rows={4}
						maxlength={4000}
						required
						placeholder={m.system_support_details_placeholder()}
						bind:value={message}
						class="min-h-24 w-full resize-y rounded-md border border-input bg-card px-3 py-2 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
					></textarea>
				</div>

				<div class="overflow-hidden rounded-lg border border-border text-xs">
					<button
						type="button"
						aria-expanded={showDiagnosticsPreview}
						class="flex w-full cursor-pointer items-center justify-between px-3 py-2 text-start text-muted-foreground transition-colors hover:text-foreground"
						onclick={() => (showDiagnosticsPreview = !showDiagnosticsPreview)}
					>
						<span class="flex items-center gap-2">
							<span class="text-[11px] font-medium">{m.system_support_diagnostics()}</span>
							{#if info}
								<span class="h-3.5 rounded border border-border px-1 py-0 font-mono text-[9px]">
									{info.version ?? m.system_support_unavailable()}
								</span>
							{/if}
						</span>
						<span>
							{showDiagnosticsPreview
								? m.system_support_hide_diagnostics()
								: m.system_support_show_diagnostics()}
						</span>
					</button>
					{#if showDiagnosticsPreview}
						<pre class="max-h-36 overflow-y-auto border-t border-border/40 bg-background/50 p-3 text-start font-mono text-[10px] text-muted-foreground" dir="ltr">{JSON.stringify(diagnosticPayload, null, 2)}</pre>
					{/if}
					{#if isLoadingInfo}
						<p class="px-3 pb-3 text-xs text-muted-foreground">{m.system_support_loading_diagnostics()}</p>
					{:else if infoError}
						<p class="px-3 pb-3 text-xs text-destructive" role="alert">
							{m.system_support_diagnostics_unavailable()} {infoError}
						</p>
					{/if}
				</div>

				{#if submitError}
					<div class="space-y-2 rounded-lg border border-destructive/20 bg-destructive/5 p-3 text-xs text-destructive" role="alert">
						<p>{m.system_support_compose_failed()} {submitError}</p>
						{#if composeUrl}
							<a
								href={composeUrl}
								target="_blank"
								rel="noreferrer"
								class="font-medium underline underline-offset-4"
							>
								{m.system_support_open_email_composer()}
							</a>
						{/if}
					</div>
				{/if}
			</form>

			<div class="flex items-center justify-between border-t border-border pt-3">
				<Button
					type="button"
					variant="outline"
					size="sm"
					onclick={() => void copyDiagnosticReport()}
				>
					{isCopied ? m.system_support_report_copied() : m.system_support_copy_report()}
				</Button>
				<div class="flex items-center gap-2">
					<Button type="button" variant="ghost" size="sm" onclick={() => (open = false)}>
						{m.common_cancel()}
					</Button>
					<Button
						type="submit"
						form="support-form"
						size="sm"
						disabled={isSubmitting || !subject.trim() || !message.trim()}
					>
						{#if isSubmitting}
							{m.system_support_sending()}
						{:else}
							{m.system_support_send_report()}
						{/if}
					</Button>
				</div>
			</div>
		{/if}
	</DialogContent>
</Dialog>
