import { desktopApi, type ConnectionConfig, type ConnectionMode, type UpdateProgress, type UpdateResult, type UserEditorInput } from '#lib/api/desktop';
import { confirmStore } from '#lib/core/confirm.svelte';
import { changeAppLocale, localeState, type AppLocale } from '#lib/core/i18n';
import { queryClient } from '#lib/core/query-client';
import type { User } from '#lib/features/auth/types';
import { userQueryKeys } from '#lib/features/users/queries';
import * as m from '#lib/paraglide/messages';
import { getLocale } from '#lib/paraglide/runtime';
import { getContext, setContext } from 'svelte';
import { toast, type ToastPosition } from '@aqardesk/ui';

export const APP_CONTEXT = Symbol('aqardesk-app');

export type AppTheme = 'light' | 'dark';
export type AppContrast = 'normal' | 'high';

export function createAppContext() {
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
	let errorToastId: number | undefined;
	let noticeToastId: number | undefined;
	let busy = $state(false);
	let booting = $state(true);
	let organizationName = $state('');
	let organizationTimezone = $state(Intl.DateTimeFormat().resolvedOptions().timeZone);
	let fullName = $state('');
	let email = $state('');
	let accountPassword = $state('');
	let updateResult = $state<UpdateResult | null>(null);
	let updateProgress = $state<UpdateProgress | null>(null);
	let updateError = $state<string | null>(null);
	let isCheckingUpdates = $state(false);
	let isDownloadingUpdate = $state(false);
	let isInstallingUpdate = $state(false);
	let isUpdateReady = $state(false);
	let theme = $state<AppTheme>('dark');
	let contrast = $state<AppContrast>('normal');
	let toastPosition = $state<ToastPosition>('top-center');
	let initialization: Promise<void> | undefined;
	let updateInitialization: Promise<void> | undefined;

	const locale = $derived.by(() => {
		localeState.revision;
		return getLocale();
	});
	const progressPercent = $derived(
		updateProgress?.total_bytes
			? Math.min(100, Math.round((updateProgress.downloaded_bytes / updateProgress.total_bytes) * 100))
			: null
	);
	const updateBusy = $derived(
		isCheckingUpdates || isDownloadingUpdate || isInstallingUpdate
	);

	async function initialize() {
		if (!initialization) {
			initialization = (async () => {
				try {
					const config = await desktopApi.getConnection();
					if (config) {
						loadConfig(config);
						configured = true;
						await connectConfigured();
					}
				} catch (cause) {
					setError(cause);
				} finally {
					booting = false;
				}
			})();
		}
		return initialization;
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
			const config = await desktopApi.setConnection({
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
			await connectConfigured();
			return true;
		} catch (cause) {
			connected = false;
			setError(cause);
			return false;
		} finally {
			busy = false;
		}
	}

	async function connectConfigured() {
		await desktopApi.connect();
		const status = await desktopApi.getSetupStatus();
		connected = true;
		needsAdmin = status.needs_admin;
		currentUser = status.current_user;
		await loadOrganizationName();
	}

	async function loadOrganizationName() {
		if (!currentUser) {
			organizationName = '';
			return;
		}
		const [name, settings] = await Promise.all([
			desktopApi.getOrganizationName(),
			desktopApi.getOrganizationSettings()
		]);
		organizationName = name;
		organizationTimezone = settings.timezone;
	}

	async function checkApi() {
		busy = true;
		clearMessages();
		try {
			await desktopApi.checkApi(serverUrl);
			setNotice(m.status_connected());
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
			currentUser = await desktopApi.createAdmin({
				organizationName,
				name: fullName,
				email,
				password: accountPassword
			});
			await loadOrganizationName();
			needsAdmin = false;
			accountPassword = '';
			setNotice(m.notification_admin_created());
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
			currentUser = await desktopApi.signIn(email, accountPassword);
			await loadOrganizationName();
			accountPassword = '';
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function addUser(input: { name: string; email: string; password: string; role: string }) {
		busy = true;
		clearMessages();
		try {
			await desktopApi.createUser(input);
			await queryClient.invalidateQueries({
				queryKey: userQueryKeys.all(currentUser?.organization_id ?? '')
			});
			setNotice(m.notification_user_added());
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		} finally {
			busy = false;
		}
	}

	async function updateUser(userId: string, input: UserEditorInput) {
		busy = true;
		clearMessages();
		try {
			await desktopApi.updateUser(userId, input);
			await queryClient.invalidateQueries({
				queryKey: userQueryKeys.all(currentUser?.organization_id ?? '')
			});
			setNotice(m.notification_user_updated());
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		} finally {
			busy = false;
		}
	}

	async function setUserActive(user: User) {
		busy = true;
		clearMessages();
		try {
			const confirmed = await confirmStore.prompt({
				title: user.is_active
					? m.users_deactivate_confirm_title()
					: m.users_activate_confirm_title(),
				description: user.is_active
					? m.users_deactivate_confirm_description({ name: user.name })
					: m.users_activate_confirm_description({ name: user.name }),
				confirmText: user.is_active ? m.users_deactivate() : m.users_activate(),
				cancelText: m.common_cancel(),
				variant: user.is_active ? 'destructive' : 'default'
			});
			if (!confirmed) return;

			await desktopApi.setUserActive(user.id, !user.is_active);
			await queryClient.invalidateQueries({
				queryKey: userQueryKeys.all(currentUser?.organization_id ?? '')
			});
		} catch (cause) {
			setError(cause);
		} finally {
			busy = false;
		}
	}

	async function signOut() {
		busy = true;
		clearMessages();
		try {
			await desktopApi.signOut();
			currentUser = null;
			organizationName = '';
			organizationTimezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		} finally {
			busy = false;
		}
	}

	async function initializeUpdates() {
		if (!updateInitialization) {
			updateInitialization = checkForUpdates(true).then(() => undefined);
		}
		return updateInitialization;
	}

	async function checkForUpdates(silent = false): Promise<UpdateResult | null> {
		if (updateBusy) return null;
		if (!silent) clearMessages();
		updateError = null;
		isCheckingUpdates = true;
		updateProgress = null;
		try {
			updateResult = await desktopApi.checkForUpdates();
			isUpdateReady = false;
			if (!silent) {
				setNotice(updateResult.has_update ? m.settings_update_available() : m.settings_up_to_date());
			}
			return updateResult;
		} catch (cause) {
			updateError = cause instanceof Error ? cause.message : String(cause);
			if (!silent) setError(cause);
			return null;
		} finally {
			isCheckingUpdates = false;
		}
	}

	async function downloadUpdate() {
		if (updateBusy) return;
		if (!updateResult?.has_update) await checkForUpdates(true);
		if (!updateResult?.has_update || updateBusy) return;
		clearMessages();
		updateError = null;
		isDownloadingUpdate = true;
		isUpdateReady = false;
		updateProgress = null;
		try {
			await desktopApi.downloadUpdate();
			setNotice(m.notification_update_ready());
			isUpdateReady = true;
		} catch (cause) {
			updateError = cause instanceof Error ? cause.message : String(cause);
			setError(cause);
		} finally {
			isDownloadingUpdate = false;
		}
	}

	async function installUpdate() {
		if (!isUpdateReady || updateBusy) return;
		clearMessages();
		updateError = null;
		isInstallingUpdate = true;
		try {
			await desktopApi.installUpdate();
		} catch (cause) {
			updateError = cause instanceof Error ? cause.message : String(cause);
			setError(cause);
			isInstallingUpdate = false;
		}
	}

	async function retryUpdate() {
		if (isUpdateReady) {
			await installUpdate();
		} else if (updateResult?.has_update) {
			await downloadUpdate();
		} else {
			await checkForUpdates();
		}
	}

	async function changeLanguage(nextLocale: AppLocale): Promise<boolean> {
		try {
			await changeAppLocale(nextLocale);
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		}
	}

	function applyTheme(nextTheme: AppTheme) {
		theme = nextTheme;
		document.documentElement.classList.toggle('dark', nextTheme === 'dark');
	}

	function initializeTheme() {
		try {
			applyTheme(localStorage.getItem('aqardesk:theme') === 'light' ? 'light' : 'dark');
			const storedContrast = localStorage.getItem('aqardesk:contrast');
			applyContrast(storedContrast === 'high' ? 'high' : 'normal');
			const storedToastPosition = localStorage.getItem('aqardesk:toast-position');
			if (isToastPosition(storedToastPosition)) toastPosition = storedToastPosition;
		} catch (cause) {
			setError(cause);
		}
	}

	function changeTheme(nextTheme: AppTheme) {
		try {
			localStorage.setItem('aqardesk:theme', nextTheme);
			applyTheme(nextTheme);
		} catch (cause) {
			setError(cause);
		}
	}

	function applyContrast(nextContrast: AppContrast) {
		contrast = nextContrast;
		document.documentElement.dataset.contrast = nextContrast;
	}

	function changeContrast(nextContrast: AppContrast) {
		try {
			localStorage.setItem('aqardesk:contrast', nextContrast);
			applyContrast(nextContrast);
		} catch (cause) {
			setError(cause);
		}
	}

	function changeToastPosition(nextPosition: ToastPosition): boolean {
		try {
			localStorage.setItem('aqardesk:toast-position', nextPosition);
			toastPosition = nextPosition;
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		}
	}

	function resetAppearance(): boolean {
		try {
			localStorage.setItem('aqardesk:theme', 'dark');
			localStorage.setItem('aqardesk:contrast', 'normal');
			localStorage.setItem('aqardesk:toast-position', 'top-center');
			applyTheme('dark');
			applyContrast('normal');
			toastPosition = 'top-center';
			return true;
		} catch (cause) {
			setError(cause);
			return false;
		}
	}

	function isToastPosition(value: string | null): value is ToastPosition {
		return (
			value === 'top-left' ||
			value === 'top-center' ||
			value === 'top-right' ||
			value === 'bottom-left' ||
			value === 'bottom-center' ||
			value === 'bottom-right'
		);
	}

	function setError(cause: unknown) {
		if (errorToastId !== undefined) toast.dismiss(errorToastId);
		errorToastId = toast.error(cause instanceof Error ? cause.message : String(cause));
	}

	function setNotice(message: string) {
		if (noticeToastId !== undefined) toast.dismiss(noticeToastId);
		noticeToastId = toast.success(message);
	}

	function updateCurrentUserProfile(name: string, email: string) {
		if (currentUser) currentUser = { ...currentUser, name, email };
	}

	function clearMessages() {
		if (errorToastId !== undefined) toast.dismiss(errorToastId);
		if (noticeToastId !== undefined) toast.dismiss(noticeToastId);
		errorToastId = undefined;
		noticeToastId = undefined;
	}

	return {
		get mode() { return mode; },
		set mode(value: ConnectionMode) { mode = value; },
		get databasePath() { return databasePath; },
		set databasePath(value: string) { databasePath = value; },
		get serverUrl() { return serverUrl; },
		set serverUrl(value: string) { serverUrl = value; },
		get host() { return host; },
		set host(value: string) { host = value; },
		get port() { return port; },
		set port(value: number) { port = value; },
		get databaseName() { return databaseName; },
		set databaseName(value: string) { databaseName = value; },
		get username() { return username; },
		set username(value: string) { username = value; },
		get password() { return password; },
		set password(value: string) { password = value; },
		get sslMode() { return sslMode; },
		set sslMode(value: string) { sslMode = value; },
		get configured() { return configured; },
		get connected() { return connected; },
		get needsAdmin() { return needsAdmin; },
		get currentUser() { return currentUser; },
		get busy() { return busy; },
		get booting() { return booting; },
		get locale() { return locale; },
		get theme() { return theme; },
		get contrast() { return contrast; },
		get toastPosition() { return toastPosition; },
		get organizationName() { return organizationName; },
		set organizationName(value: string) { organizationName = value; },
		get organizationTimezone() { return organizationTimezone; },
		set organizationTimezone(value: string) { organizationTimezone = value; },
		get fullName() { return fullName; },
		set fullName(value: string) { fullName = value; },
		get email() { return email; },
		set email(value: string) { email = value; },
		get accountPassword() { return accountPassword; },
		set accountPassword(value: string) { accountPassword = value; },
		get updateResult() { return updateResult; },
		get updateProgress() { return updateProgress; },
		get progressPercent() { return progressPercent; },
		get updateError() { return updateError; },
		get isCheckingUpdates() { return isCheckingUpdates; },
		get isDownloadingUpdate() { return isDownloadingUpdate; },
		get isInstallingUpdate() { return isInstallingUpdate; },
		get isUpdateReady() { return isUpdateReady; },
		get updateBusy() { return updateBusy; },
		set updateProgress(value: UpdateProgress | null) { updateProgress = value; },
		initialize,
		initializeUpdates,
		saveAndConnect,
		checkApi,
		createInitialAdmin,
		signIn,
		addUser,
		updateUser,
		setUserActive,
		signOut,
		checkForUpdates,
		downloadUpdate,
		installUpdate,
		retryUpdate,
		changeLanguage,
		initializeTheme,
		changeTheme,
		changeContrast,
		changeToastPosition,
		resetAppearance,
		updateCurrentUserProfile,
		setError,
		clearMessages
	};
}

export type AppContext = ReturnType<typeof createAppContext>;

declare global {
	interface Window {
		__aqardeskAppContext?: AppContext;
	}
}

export function setAppContext(context: AppContext) {
	setContext(APP_CONTEXT, context);
	if (typeof window !== 'undefined') {
		window.__aqardeskAppContext = context;
	}
}

export function getAppContext() {
	const context = getContext<AppContext | undefined>(APP_CONTEXT);
	if (context) return context;

	if (typeof window !== 'undefined' && window.__aqardeskAppContext) {
		return window.__aqardeskAppContext;
	}

	throw new Error('AqarDesk app context is missing');
}
