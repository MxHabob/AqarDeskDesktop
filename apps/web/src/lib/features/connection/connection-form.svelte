<script lang="ts">
	import { Database, Globe2, Server, ShieldCheck } from '@lucide/svelte';
	import {
		Button,
		Card,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue
	} from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';
	import FormField from '#lib/shared/components/form-field.svelte';

	type Props = {
		mode: ConnectionMode;
		databasePath: string;
		serverUrl: string;
		host: string;
		port: number;
		databaseName: string;
		username: string;
		password: string;
		sslMode: string;
		busy: boolean;
		onSubmit: (event: SubmitEvent) => void;
		onApiCheck: () => void;
		onModeChange: (mode: ConnectionMode) => void;
		onDatabasePathChange: (value: string) => void;
		onServerUrlChange: (value: string) => void;
		onHostChange: (value: string) => void;
		onPortChange: (value: number) => void;
		onDatabaseNameChange: (value: string) => void;
		onUsernameChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onSslModeChange: (value: string) => void;
	};

	let {
		mode,
		databasePath,
		serverUrl,
		host,
		port,
		databaseName,
		username,
		password,
		sslMode,
		busy,
		onSubmit,
		onApiCheck,
		onModeChange,
		onDatabasePathChange,
		onServerUrlChange,
		onHostChange,
		onPortChange,
		onDatabaseNameChange,
		onUsernameChange,
		onPasswordChange,
		onSslModeChange
	}: Props = $props();
</script>

<Card class="rounded-xl bg-card p-5 shadow-card ring-border/50 sm:p-6">
	<form class="grid gap-5" onsubmit={onSubmit}>
		<div class="grid gap-2 sm:grid-cols-3">
			<Button
				variant="outline"
				class={mode === 'local'
					? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-primary bg-accent p-3 text-start text-accent-foreground'
					: 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'}
				onclick={() => onModeChange('local')}
			>
				<Database size={18} />
				<strong>{m.connection_local()}</strong>
				<small class="whitespace-normal text-start text-xs font-normal">
					{m.connection_local_description()}
				</small>
			</Button>
			<Button
				variant="outline"
				class={mode === 'direct'
					? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-primary bg-accent p-3 text-start text-accent-foreground'
					: 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'}
				onclick={() => onModeChange('direct')}
			>
				<Server size={18} />
				<strong>{m.connection_direct()}</strong>
				<small class="whitespace-normal text-start text-xs font-normal">
					{m.connection_direct_description()}
				</small>
			</Button>
			<Button
				variant="outline"
				class={mode === 'api'
					? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-primary bg-accent p-3 text-start text-accent-foreground'
					: 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'}
				onclick={() => onModeChange('api')}
			>
				<Globe2 size={18} />
				<strong>{m.connection_api()}</strong>
				<small class="whitespace-normal text-start text-xs font-normal">
					{m.connection_api_description()}
				</small>
			</Button>
		</div>

		{#if mode === 'local'}
			<FormField label={m.connection_database_file()} id="database-path">
				<Input
					id="database-path"
					value={databasePath}
					oninput={(event) => onDatabasePathChange(event.currentTarget.value)}
					required
					dir="ltr"
					autocomplete="off"
					class="rounded-md"
				/>
			</FormField>
		{:else if mode === 'api'}
			<FormField label={m.connection_server_url()} id="server-url">
				<Input
					id="server-url"
					type="url"
					value={serverUrl}
					oninput={(event) => onServerUrlChange(event.currentTarget.value)}
					required
					dir="ltr"
					placeholder="https://api.example.com"
					autocomplete="url"
					class="rounded-md"
				/>
			</FormField>
		{:else}
			<div class="grid gap-4 sm:grid-cols-2">
				<FormField label={m.connection_host()} id="database-host">
					<Input
						id="database-host"
						value={host}
						oninput={(event) => onHostChange(event.currentTarget.value)}
						required
						dir="ltr"
						autocomplete="off"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_port()} id="database-port">
					<Input
						id="database-port"
						type="number"
						value={port}
						oninput={(event) => onPortChange(Number(event.currentTarget.value))}
						min="1"
						max="65535"
						required
						dir="ltr"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_database_name()} id="database-name">
					<Input
						id="database-name"
						value={databaseName}
						oninput={(event) => onDatabaseNameChange(event.currentTarget.value)}
						required
						dir="ltr"
						autocomplete="off"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_username()} id="database-username">
					<Input
						id="database-username"
						value={username}
						oninput={(event) => onUsernameChange(event.currentTarget.value)}
						required
						dir="ltr"
						autocomplete="username"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_password()} id="database-password">
					<Input
						id="database-password"
						type="password"
						value={password}
						oninput={(event) => onPasswordChange(event.currentTarget.value)}
						dir="ltr"
						autocomplete="new-password"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_ssl_mode()} id="ssl-mode">
					<Select
						type="single"
						value={sslMode}
						onValueChange={onSslModeChange}
					>
						<SelectTrigger id="ssl-mode" class="h-9 w-full rounded-md text-sm">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="prefer">prefer</SelectItem>
							<SelectItem value="require">require</SelectItem>
							<SelectItem value="disable">disable</SelectItem>
						</SelectContent>
					</Select>
				</FormField>
			</div>
			<p class="flex items-start gap-2 text-xs leading-relaxed text-muted-foreground">
				<ShieldCheck size={15} class="mt-0.5 shrink-0 text-foreground" />
				{m.connection_security_note()}
			</p>
		{/if}

		<div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
			{#if mode === 'api'}
				<Button
					variant="outline"
					class="h-10 rounded-md"
					type="button"
					disabled={busy}
					onclick={onApiCheck}
				>
					{m.common_retry()}
				</Button>
			{/if}
			<Button class="h-10 rounded-md px-5" type="submit" disabled={busy}>
				{busy ? m.common_processing() : m.common_connect()}
			</Button>
		</div>
	</form>
</Card>
