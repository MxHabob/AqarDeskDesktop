<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	type ConnectionMode = 'local' | 'direct' | 'api';
	type ConnectionConfig = {
		mode: ConnectionMode;
		local?: { database_path: string } | null;
		direct?: {
			host: string;
			port: number;
			database_name: string;
			username: string;
			ssl_mode?: string | null;
		} | null;
		api?: { server_url: string } | null;
	};
	type User = {
		id: string;
		name: string;
		email: string;
		role: string;
		is_active: boolean;
		created_at: string;
	};
	type UpdateResult = {
		current_version: string;
		latest_version: string;
		has_update: boolean;
		release_notes: string | null;
	};

	let mode = $state<ConnectionMode>('local');
	let databasePath = $state('aqardesk.sqlite');
	let serverUrl = $state('');
	let host = $state('localhost');
	let port = $state(5432);
	let databaseName = $state('aqardesk');
	let username = $state('');
	let password = $state('');
	let sslMode = $state('prefer');
	let configured = $state(false);
	let connected = $state(false);
	let needsAdmin = $state(false);
	let currentUser = $state<User | null>(null);
	let users = $state<User[]>([]);
	let error = $state('');
	let notice = $state('');
	let busy = $state(false);
	let organizationName = $state('');
	let fullName = $state('');
	let email = $state('');
	let accountPassword = $state('');
	let newUserName = $state('');
	let newUserEmail = $state('');
	let newUserPassword = $state('');
	let newUserRole = $state('staff');
	let updateResult = $state<UpdateResult | null>(null);
	let updateProgress = $state<{ downloaded_bytes: number; total_bytes: number | null } | null>(
		null
	);
	let unlistenProgress: UnlistenFn | undefined;

	onMount(() => {
		void initialize();
		void listen<{ downloaded_bytes: number; total_bytes: number | null }>(
			'desktop-update-progress',
			(event) => {
				updateProgress = event.payload;
			}
		).then((unlisten) => {
			unlistenProgress = unlisten;
		});
		return () => unlistenProgress?.();
	});

	async function initialize() {
		try {
			const config = await invoke<ConnectionConfig | null>('connection_get_config');
			if (!config) return;
			loadConfig(config);
			configured = true;
			await connect();
		} catch (cause) {
			setError(cause);
		}
	}

	function loadConfig(config: ConnectionConfig) {
		mode = config.mode;
		databasePath = config.local?.database_path ?? 'aqardesk.sqlite';
		serverUrl = config.api?.server_url ?? '';
		host = config.direct?.host ?? 'localhost';
		port = config.direct?.port ?? 5432;
		databaseName = config.direct?.database_name ?? 'aqardesk';
		username = config.direct?.username ?? '';
		sslMode = config.direct?.ssl_mode ?? 'prefer';
		password = '';
	}

	async function saveAndConnect(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		clearMessages();
		try {
			const config = await invoke<ConnectionConfig>('connection_set_config', {
				mode,
				databasePath: mode === 'local' ? databasePath : null,
				serverUrl: mode === 'api' ? serverUrl : null,
				host: mode === 'direct' ? host : null,
				port: mode === 'direct' ? Number(port) : null,
				databaseName: mode === 'direct' ? databaseName : null,
				username: mode === 'direct' ? username : null,
				password: mode === 'direct' ? password : null,
				sslMode: mode === 'direct' ? sslMode : null
			});
			loadConfig(config);
			configured = true;
			await connect();
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function connect() {
		busy = true;
		clearMessages();
		try {
			await invoke('connection_connect');
			connected = true;
			const status = await invoke<{ needs_admin: boolean; current_user: User | null }>(
				'auth_get_setup_status'
			);
			needsAdmin = status.needs_admin;
			currentUser = status.current_user;
			if (currentUser?.role === 'admin') await loadUsers();
		} catch (cause) {
			connected = false;
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function checkApi() {
		busy = true;
		clearMessages();
		try {
			await invoke('connection_check_api', { serverUrl });
			notice = 'تم الاتصال بخادم API بنجاح.';
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function createInitialAdmin(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		clearMessages();
		try {
			currentUser = await invoke<User>('auth_bootstrap_admin', {
				organizationName,
				name: fullName,
				email,
				password: accountPassword
			});
			needsAdmin = false;
			accountPassword = '';
			await loadUsers();
			notice = 'تم إنشاء المؤسسة وحساب المدير.';
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function signIn(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		clearMessages();
		try {
			currentUser = await invoke<User>('auth_login', { email, password: accountPassword });
			accountPassword = '';
			if (currentUser.role === 'admin') await loadUsers();
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function loadUsers() {
		users = await invoke<User[]>('users_list');
	}

	async function addUser(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		clearMessages();
		try {
			await invoke('users_create', {
				name: newUserName,
				email: newUserEmail,
				password: newUserPassword,
				role: newUserRole
			});
			newUserName = '';
			newUserEmail = '';
			newUserPassword = '';
			await loadUsers();
			notice = 'تمت إضافة المستخدم.';
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function setUserActive(user: User) {
		clearMessages();
		try {
			await invoke('users_set_active', { userId: user.id, isActive: !user.is_active });
			await loadUsers();
		} catch (cause) {
			setError(cause);
		}
	}

	async function signOut() {
		await invoke('auth_logout');
		currentUser = null;
		users = [];
	}

	async function checkForUpdates() {
		busy = true;
		clearMessages();
		updateProgress = null;
		try {
			updateResult = await invoke<UpdateResult>('system_check_for_updates');
			notice = updateResult.has_update
				? `يتوفر تحديث جديد: ${updateResult.latest_version}`
				: `التطبيق محدّث (${updateResult.current_version}).`;
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function downloadUpdate() {
		busy = true;
		clearMessages();
		updateProgress = null;
		try {
			await invoke<string>('system_download_update');
			notice = 'اكتمل تنزيل التحديث والتحقق من توقيعه. يمكنك تثبيته الآن.';
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function installUpdate() {
		busy = true;
		clearMessages();
		try {
			await invoke('system_install_update');
		} catch (cause) {
			setError(cause);
			busy = false;
		}
	}

	function setError(cause: unknown) {
		error = cause instanceof Error ? cause.message : String(cause);
	}

	function clearMessages() {
		error = '';
		notice = '';
	}
</script>

<svelte:head>
	<title>AqarDesk | إدارة العقارات</title>
	<meta name="description" content="إدارة العقارات والمستأجرين والعقود" />
</svelte:head>

<main dir="rtl" lang="ar">
	<header>
		<div>
			<p class="eyebrow">نظام إدارة العقارات</p>
			<h1>AqarDesk</h1>
		</div>
		{#if currentUser}
			<div class="account">
				<span>{currentUser.name}</span>
				<button class="subtle" onclick={signOut}>تسجيل الخروج</button>
			</div>
		{/if}
	</header>

	{#if error}
		<p class="message error" role="alert">{error}</p>
	{/if}
	{#if notice}
		<p class="message success" role="status">{notice}</p>
	{/if}

	{#if !configured || !connected}
		<section class="panel setup">
			<p class="eyebrow">إعداد أولي</p>
			<h2>اختر طريقة الاتصال</h2>
			<p class="muted">
				تحفظ إعدادات الاتصال على هذا الجهاز. كلمات مرور PostgreSQL تحفظ في مخزن بيانات الاعتماد
				لنظام التشغيل، ولا تحفظ داخل ملف الإعدادات.
			</p>

			<div class="mode-picker" role="group" aria-label="طريقة الاتصال">
				<button class:chosen={mode === 'local'} onclick={() => (mode = 'local')}>
					<strong>قاعدة محلية</strong><span>SQLite على هذا الجهاز</span>
				</button>
				<button class:chosen={mode === 'direct'} onclick={() => (mode = 'direct')}>
					<strong>PostgreSQL</strong><span>اتصال مباشر بقاعدة مشتركة</span>
				</button>
				<button class:chosen={mode === 'api'} onclick={() => (mode = 'api')}>
					<strong>خادم API</strong><span>اتصال بخادم مُعد مسبقًا</span>
				</button>
			</div>

			<form onsubmit={saveAndConnect}>
				{#if mode === 'local'}
					<label>مسار ملف قاعدة البيانات
						<input bind:value={databasePath} required placeholder="aqardesk.sqlite أو مسار كامل" />
					</label>
				{:else if mode === 'direct'}
					<div class="field-grid">
						<label>المضيف<input bind:value={host} required placeholder="localhost" /></label>
						<label>المنفذ<input type="number" bind:value={port} min="1" max="65535" required /></label>
						<label>اسم قاعدة البيانات<input bind:value={databaseName} required /></label>
						<label>اسم المستخدم<input bind:value={username} required autocomplete="username" /></label>
						<label>كلمة المرور<input type="password" bind:value={password} autocomplete="new-password" placeholder="اتركه فارغًا لاستخدام المحفوظ" /></label>
						<label>SSL
							<select bind:value={sslMode}>
								<option value="prefer">تلقائي (prefer)</option>
								<option value="require">مطلوب</option>
								<option value="verify-full">تحقق كامل</option>
								<option value="disable">تعطيل</option>
							</select>
						</label>
					</div>
				{:else}
					<label>عنوان خادم API<input type="url" bind:value={serverUrl} required placeholder="https://server.example.com" /></label>
					<button class="subtle" type="button" onclick={checkApi} disabled={busy}>اختبار الاتصال</button>
				{/if}
				<div class="actions">
					<button class="primary" type="submit" disabled={busy}>{busy ? 'جارٍ الاتصال…' : 'حفظ والاتصال'}</button>
				</div>
			</form>
		</section>
	{:else if needsAdmin}
		<section class="panel narrow">
			<p class="eyebrow">إعداد المؤسسة</p>
			<h2>إنشاء حساب المدير الأول</h2>
			<p class="muted">سيكون هذا الحساب مسؤولًا عن إدارة حسابات المستخدمين.</p>
			<form onsubmit={createInitialAdmin}>
				<label>اسم المؤسسة<input bind:value={organizationName} required /></label>
				<label>الاسم الكامل<input bind:value={fullName} required autocomplete="name" /></label>
				<label>البريد الإلكتروني<input type="email" bind:value={email} required autocomplete="email" /></label>
				<label>كلمة المرور<input type="password" bind:value={accountPassword} required minlength="12" autocomplete="new-password" /></label>
				<button class="primary" type="submit" disabled={busy}>إنشاء المدير وبدء الاستخدام</button>
			</form>
		</section>
	{:else if !currentUser}
		<section class="panel narrow">
			<p class="eyebrow">مرحبًا بعودتك</p>
			<h2>تسجيل الدخول</h2>
			<form onsubmit={signIn}>
				<label>البريد الإلكتروني<input type="email" bind:value={email} required autocomplete="username" /></label>
				<label>كلمة المرور<input type="password" bind:value={accountPassword} required autocomplete="current-password" /></label>
				<button class="primary" type="submit" disabled={busy}>تسجيل الدخول</button>
			</form>
			<button class="subtle" onclick={() => (connected = false)}>تغيير اتصال قاعدة البيانات</button>
		</section>
	{:else}
		<section class="welcome panel">
			<div>
				<p class="eyebrow">لوحة التحكم</p>
				<h2>أهلًا، {currentUser.name}</h2>
				<p class="muted">تم تجهيز قاعدة البيانات لإدارة العقارات والوحدات والمستأجرين والعقود والمدفوعات.</p>
			</div>
			<button class="subtle" onclick={() => (connected = false)}>إعدادات الاتصال</button>
		</section>

		{#if currentUser.role === 'admin'}
			<section class="panel">
				<div class="section-heading">
					<div><p class="eyebrow">إدارة الوصول</p><h2>المستخدمون</h2></div>
					<button class="subtle" onclick={loadUsers}>تحديث القائمة</button>
				</div>
				<form class="user-form" onsubmit={addUser}>
					<label>الاسم<input bind:value={newUserName} required /></label>
					<label>البريد الإلكتروني<input type="email" bind:value={newUserEmail} required /></label>
					<label>كلمة المرور<input type="password" bind:value={newUserPassword} minlength="12" required /></label>
					<label>الدور
						<select bind:value={newUserRole}>
							<option value="staff">موظف</option>
							<option value="readonly">قراءة فقط</option>
							<option value="admin">مدير</option>
						</select>
					</label>
					<button class="primary" type="submit" disabled={busy}>إضافة مستخدم</button>
				</form>
				<div class="table-wrap">
					<table>
						<thead><tr><th>الاسم</th><th>البريد الإلكتروني</th><th>الدور</th><th>الحالة</th><th></th></tr></thead>
						<tbody>
							{#each users as user (user.id)}
								<tr>
									<td>{user.name}</td><td dir="ltr">{user.email}</td>
									<td>{user.role === 'admin' ? 'مدير' : user.role === 'readonly' ? 'قراءة فقط' : 'موظف'}</td>
									<td>{user.is_active ? 'نشط' : 'موقوف'}</td>
									<td>
										{#if user.id !== currentUser.id}
											<button class="subtle" onclick={() => setUserActive(user)}>{user.is_active ? 'إيقاف' : 'تفعيل'}</button>
										{/if}
									</td>
								</tr>
							{:else}
								<tr><td colspan="5" class="muted">لا يوجد مستخدمون.</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{/if}

		<section class="panel updates">
			<div><p class="eyebrow">التطبيق</p><h2>التحديثات</h2><p class="muted">تُنزل التحديثات الموقّعة فقط من قناة AqarDesk.</p></div>
			<div class="actions">
				<button class="subtle" onclick={checkForUpdates} disabled={busy}>التحقق من التحديثات</button>
				{#if updateResult?.has_update}
					<button class="primary" onclick={downloadUpdate} disabled={busy}>تنزيل {updateResult.latest_version}</button>
				{/if}
				{#if updateProgress}
					<span class="muted">
						{updateProgress.downloaded_bytes} / {updateProgress.total_bytes ?? '…'} بايت
					</span>
				{/if}
				{#if updateResult?.has_update && !busy && !updateProgress}
					<button class="primary" onclick={installUpdate}>تثبيت وإعادة التشغيل</button>
				{/if}
			</div>
		</section>
	{/if}
</main>

<style>
	:global(*) { box-sizing: border-box; }
	:global(body) { margin: 0; background: #f4f6f8; color: #17212b; font-family: "Segoe UI", Tahoma, sans-serif; }
	main { max-width: 1080px; margin: 0 auto; padding: 34px 22px 64px; }
	header, .account, .section-heading, .welcome, .updates, .actions { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
	header { margin-bottom: 28px; }
	h1, h2, p { margin: 0; }
	h1 { font-size: 30px; letter-spacing: -.04em; }
	h2 { font-size: 21px; margin: 3px 0 8px; }
	.eyebrow { color: #087e70; font-size: 12px; font-weight: 700; margin-bottom: 6px; }
	.panel { background: white; border: 1px solid #e0e6ea; border-radius: 14px; padding: 24px; margin-top: 18px; box-shadow: 0 5px 20px #132a3b0a; }
	.setup { max-width: 780px; margin-inline: auto; }
	.narrow { max-width: 500px; margin-inline: auto; }
	.muted { color: #667580; font-size: 14px; line-height: 1.7; }
	.mode-picker { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin: 22px 0; }
	.mode-picker button { text-align: right; display: grid; gap: 7px; border: 1px solid #dce4e8; background: #fff; padding: 14px; border-radius: 10px; cursor: pointer; color: inherit; }
	.mode-picker button span { color: #667580; font-size: 12px; }
	.mode-picker button.chosen { border-color: #078878; box-shadow: 0 0 0 2px #07887818; background: #f5fbfa; }
	form { display: grid; gap: 14px; margin-top: 20px; }
	label { display: grid; gap: 6px; font-size: 13px; font-weight: 600; }
	input, select { width: 100%; min-height: 40px; border: 1px solid #ccd6dc; background: white; border-radius: 7px; padding: 8px 10px; color: #17212b; font: inherit; }
	input:focus, select:focus { outline: 2px solid #087e7040; border-color: #087e70; }
	.field-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 14px; }
	button { border: 0; border-radius: 7px; min-height: 38px; padding: 8px 14px; font: inherit; font-size: 13px; font-weight: 700; cursor: pointer; }
	button:disabled { opacity: .6; cursor: wait; }
	.primary { background: #087e70; color: white; }
	.subtle { background: #edf2f4; color: #30434e; }
	.actions { justify-content: flex-start; flex-wrap: wrap; margin-top: 5px; }
	.account { color: #43535d; font-size: 14px; }
	.message { margin: 12px 0; border-radius: 8px; padding: 12px 14px; font-size: 14px; }
	.error { color: #8e2525; border: 1px solid #e7b6b6; background: #fff2f2; }
	.success { color: #126049; border: 1px solid #a6d9c8; background: #effaf5; }
	.user-form { grid-template-columns: repeat(5, minmax(0, 1fr)); align-items: end; }
	.table-wrap { overflow-x: auto; margin-top: 18px; }
	table { border-collapse: collapse; width: 100%; font-size: 13px; }
	th, td { padding: 11px 9px; border-bottom: 1px solid #e8edef; text-align: right; white-space: nowrap; }
	th { color: #667580; font-size: 12px; }
	.updates { align-items: flex-start; }
	@media (max-width: 720px) {
		main { padding: 22px 14px 40px; }
		.mode-picker, .field-grid, .user-form { grid-template-columns: 1fr; }
		.welcome, .updates { align-items: flex-start; flex-direction: column; }
		.panel { padding: 18px; }
	}
</style>
