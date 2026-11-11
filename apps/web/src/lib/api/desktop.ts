import { invoke } from '@tauri-apps/api/core';
import type {
	Property,
	PropertyInput,
	PropertyUnit,
	PropertyUnitInput,
	AuditEntry,
	ContractTemplate,
	ContractTemplateInput,
	CurrencySettings,
	OrganizationSettings,
	OrganizationSettingsInput,
	Payment,
	PaymentInput,
	ProfileSettings,
	ProfileSettingsInput,
	Tenant,
	TenantInput
} from '@aqardesk/sdk';
import type { User } from '#lib/features/auth/types';

export type ConnectionMode = 'local' | 'direct' | 'api';

export type CurrencyDetailsInput = {
	name: string;
	name_ar?: string | null;
	symbol: string;
	symbol_position: 'prefix' | 'suffix';
	decimal_digits: number;
};

export type NewCurrencyInput = CurrencyDetailsInput & {
	code: string;
	enable_for_organization: boolean;
};

export type CurrencyEditorInput = NewCurrencyInput;

export type UserEditorInput = {
	name: string;
	email: string;
	role: 'admin' | 'staff' | 'readonly';
	password?: string;
};

export type ConnectionConfig = {
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

export type UpdateResult = {
	current_version: string;
	latest_version: string;
	has_update: boolean;
	release_notes: string | null;
};

export type UpdateProgress = { downloaded_bytes: number; total_bytes: number | null };

export type LicenseState =
	| 'unavailable'
	| 'pending'
	| 'trial'
	| 'active'
	| 'expiring'
	| 'offline_grace'
	| 'deactivation_pending'
	| 'expired'
	| 'suspended'
	| 'revoked'
	| 'invalid';

export type LicenseLimit = {
	key: string;
	max_value: number | null;
	unit: string;
};

export type LicenseStateDto = {
	state: LicenseState;
	product: string;
	plan: string | null;
	plan_name: string | null;
	organization_id: string | null;
	valid_until: string | null;
	days_until_expiry: number | null;
	trial_days_remaining: number;
	is_operational: boolean;
	is_restricted: boolean;
	is_configured: boolean;
	configuration_error: string | null;
	last_error: string | null;
	installation_id: string;
	last_sync_at: string | null;
	enabled_features: string[];
	trial_features: string[];
	limits: LicenseLimit[];
	app_version: string;
};

export type DashboardSummary = {
	active_properties_count: number;
	units_count: number;
	available_units_count: number;
	tenants_count: number;
	posted_payments_count: number;
	payment_months: string[];
	monthly_payments: {
		month: string;
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		payment_count: number;
	}[];
	payments_by_currency: {
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		payment_count: number;
	}[];
	unit_status_counts: { status: string; count: number }[];
	payment_status_counts: { status: string; count: number }[];
};

export type DatabaseAdminStatus = {
	mode: ConnectionMode;
	database_path: string | null;
	file_size_bytes: number | null;
	modified_at_unix: number | null;
	supports_file_management: boolean;
	remote_database: string | null;
};

export type DatabaseBackup = {
	name: string;
	path: string;
	size_bytes: number;
	modified_at_unix: number;
};

export type DatabaseOperationResult = {
	path: string;
	size_bytes: number;
};

export const desktopApi = {
	getConnection: () => invoke<ConnectionConfig | null>('platform_connection_get_config'),
	setConnection: (input: {
		mode: ConnectionMode;
		databasePath?: string | null;
		serverUrl?: string | null;
		host?: string | null;
		port?: number | null;
		databaseName?: string | null;
		username?: string | null;
		password?: string | null;
		sslMode?: string | null;
	}) => invoke<ConnectionConfig>('platform_connection_set_config', { input }),
	connect: () => invoke<void>('platform_connection_connect'),
	getDatabaseAdminStatus: () => invoke<DatabaseAdminStatus>('database_admin_get_status'),
	listDatabaseBackups: () => invoke<DatabaseBackup[]>('database_admin_list_backups'),
	createDatabaseBackup: (destinationPath: string) =>
		invoke<DatabaseOperationResult>('database_admin_backup', { destinationPath }),
	restoreDatabaseBackup: (sourcePath: string) =>
		invoke<DatabaseOperationResult>('database_admin_restore', { sourcePath }),
	checkDatabaseIntegrity: () => invoke<string>('database_admin_check_integrity'),
	changeDatabaseLocation: (destinationPath: string) =>
		invoke<DatabaseOperationResult>('database_admin_change_location', { destinationPath }),
	checkApi: (serverUrl: string) => invoke<void>('platform_connection_check_api', { serverUrl }),
	getSetupStatus: () =>
		invoke<{ needs_admin: boolean; current_user: User | null }>('platform_auth_get_setup_status'),
	getLicenseState: (organizationId: string) =>
		invoke<LicenseStateDto>('license_get_state', { organizationId }),
	activateLicense: (productKey: string) =>
		invoke<LicenseStateDto>('license_activate_key', { productKey }),
	startLicenseTrial: () => invoke<LicenseStateDto>('license_start_trial'),
	refreshLicense: () => invoke<LicenseStateDto>('license_refresh'),
	deactivateLicense: () => invoke<LicenseStateDto>('license_deactivate'),
	createAdmin: (input: {
		organizationName: string;
		name: string;
		email: string;
		password: string;
	}) => invoke<User>('platform_auth_bootstrap_admin', input),
	signIn: (email: string, password: string) =>
		invoke<User>('platform_auth_login', { email, password }),
	signOut: () => invoke<void>('platform_auth_logout'),
	getOrganizationName: () => invoke<string>('platform_organization_get_name'),
	getOrganizationSettings: () => invoke<OrganizationSettings>('organization_get_settings'),
	updateOrganizationSettings: (input: OrganizationSettingsInput) =>
		invoke<OrganizationSettings>('organization_update_settings', { input }),
	listCurrencies: () => invoke<CurrencySettings[]>('currencies_list'),
	createCurrency: (input: NewCurrencyInput) => invoke<void>('currencies_create', { input }),
	updateCurrency: (currencyCode: string, input: CurrencyDetailsInput) =>
		invoke<void>('currencies_update', { currencyCode, input }),
	setOrganizationCurrencyEnabled: (currencyCode: string, isEnabled: boolean) =>
		invoke<void>('currencies_set_organization_enabled', { currencyCode, isEnabled }),
	getProfileSettings: () => invoke<ProfileSettings>('profile_get_settings'),
	updateProfileSettings: (input: ProfileSettingsInput) =>
		invoke<ProfileSettings>('profile_update_settings', { input }),
	listAuditEvents: (limit = 50, offset = 0) =>
		invoke<AuditEntry[]>('audit_list', { limit, offset }),
	listContractTemplates: () => invoke<ContractTemplate[]>('contract_templates_list'),
	createContractTemplate: (input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_create', { input }),
	updateContractTemplate: (templateId: string, input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_update', { templateId, input }),
	setContractTemplateActive: (templateId: string, isActive: boolean) =>
		invoke<void>('contract_templates_set_active', { templateId, isActive }),
	getDashboardSummary: (months: number) => invoke<DashboardSummary>('dashboard_get_summary', { months }),
	listUsers: () => invoke<User[]>('platform_users_list'),
	createUser: (input: { name: string; email: string; password: string; role: string }) =>
		invoke<void>('platform_users_create', input),
	updateUser: (userId: string, input: UserEditorInput) =>
		invoke<User>('platform_users_update', { userId, ...input }),
	setUserActive: (userId: string, isActive: boolean) =>
		invoke<void>('platform_users_set_active', { userId, isActive }),
	listProperties: () => invoke<Property[]>('platform_properties_list'),
	createProperty: (input: PropertyInput) =>
		invoke<Property>('platform_properties_create', { input }),
	updateProperty: (propertyId: string, input: PropertyInput) =>
		invoke<Property>('platform_properties_update', { propertyId, input }),
	archiveProperty: (propertyId: string) =>
		invoke<void>('platform_properties_archive', { propertyId }),
	listUnits: () => invoke<PropertyUnit[]>('platform_units_list', { propertyId: null }),
	createUnit: (input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_create', { input }),
	updateUnit: (unitId: string, input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_update', { unitId, input }),
	archiveUnit: (unitId: string) => invoke<void>('platform_units_archive', { unitId }),
	listTenants: () => invoke<Tenant[]>('platform_tenants_list'),
	createTenant: (input: TenantInput) =>
		invoke<Tenant>('platform_tenants_create', { input }),
	updateTenant: (tenantId: string, input: TenantInput) =>
		invoke<Tenant>('platform_tenants_update', { tenantId, input }),
	archiveTenant: (tenantId: string) => invoke<void>('platform_tenants_archive', { tenantId }),
	listPayments: () => invoke<Payment[]>('platform_payments_list'),
	recordPayment: (input: PaymentInput) =>
		invoke<Payment>('platform_payments_record', { input }),
	voidPayment: (paymentId: string) => invoke<void>('platform_payments_void', { paymentId }),
	checkForUpdates: () => invoke<UpdateResult>('system_check_for_updates'),
	downloadUpdate: () => invoke<string>('system_download_update'),
	installUpdate: () => invoke<void>('system_install_update')
};
