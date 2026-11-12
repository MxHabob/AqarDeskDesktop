import * as m from '#lib/paraglide/messages';

export type Permission =
	| 'view_properties'
	| 'create_properties'
	| 'update_properties'
	| 'archive_properties'
	| 'view_owners'
	| 'create_owners'
	| 'update_owners'
	| 'view_units'
	| 'create_units'
	| 'update_units'
	| 'archive_units'
	| 'view_tenants'
	| 'create_tenants'
	| 'update_tenants'
	| 'archive_tenants'
	| 'view_payments'
	| 'record_payments'
	| 'void_payments'
	| 'manage_administration';

export type AccessRole = 'admin' | 'staff' | 'readonly';
export type BusinessView = 'properties' | 'units' | 'tenants' | 'owners' | 'payments';

export type PermissionOption = {
	key: Permission;
	label: () => string;
};

export type PermissionGroup = {
	label: () => string;
	permissions: PermissionOption[];
};

function option(key: Permission, action: () => string, area: () => string): PermissionOption {
	return { key, label: () => `${action()} ${area()}` };
}

export const permissionGroups: PermissionGroup[] = [
	{
		label: m.nav_properties,
		permissions: [
			option('view_properties', m.settings_permissions_view, m.nav_properties),
			option('create_properties', m.settings_permissions_create, m.nav_properties),
			option('update_properties', m.settings_permissions_update, m.nav_properties),
			option('archive_properties', m.settings_permissions_archive, m.nav_properties)
		]
	},
	{
		label: m.nav_owners,
		permissions: [
			option('view_owners', m.settings_permissions_view, m.nav_owners),
			option('create_owners', m.settings_permissions_create, m.nav_owners),
			option('update_owners', m.settings_permissions_update, m.nav_owners)
		]
	},
	{
		label: m.nav_units,
		permissions: [
			option('view_units', m.settings_permissions_view, m.nav_units),
			option('create_units', m.settings_permissions_create, m.nav_units),
			option('update_units', m.settings_permissions_update, m.nav_units),
			option('archive_units', m.settings_permissions_archive, m.nav_units)
		]
	},
	{
		label: m.nav_tenants,
		permissions: [
			option('view_tenants', m.settings_permissions_view, m.nav_tenants),
			option('create_tenants', m.settings_permissions_create, m.nav_tenants),
			option('update_tenants', m.settings_permissions_update, m.nav_tenants),
			option('archive_tenants', m.settings_permissions_archive, m.nav_tenants)
		]
	},
	{
		label: m.nav_payments,
		permissions: [
			option('view_payments', m.settings_permissions_view, m.nav_payments),
			option('record_payments', m.settings_permissions_record, m.nav_payments),
			option('void_payments', m.settings_permissions_void, m.nav_payments)
		]
	}
];

export const permissionDependencies: Partial<Record<Permission, readonly Permission[]>> = {
	create_properties: ['view_properties'],
	update_properties: ['view_properties'],
	archive_properties: ['view_properties'],
	create_owners: ['view_owners'],
	update_owners: ['view_owners'],
	create_units: ['view_units', 'view_properties'],
	update_units: ['view_units', 'view_properties'],
	archive_units: ['view_units'],
	create_tenants: ['view_tenants'],
	update_tenants: ['view_tenants'],
	archive_tenants: ['view_tenants'],
	record_payments: ['view_payments', 'view_tenants'],
	void_payments: ['view_payments']
};

const businessPermissions = permissionGroups.flatMap((group) =>
	group.permissions.map((permission) => permission.key)
);

const permissionLabels = Object.fromEntries(
	permissionGroups.flatMap((group) =>
		group.permissions.map((permission) => [permission.key, permission.label])
	)
) as Record<Exclude<Permission, 'manage_administration'>, () => string>;

export function permissionLabel(permission: Permission): string {
	return permission === 'manage_administration'
		? m.settings_permissions_manage_administration()
		: permissionLabels[permission]();
}

export function isPermission(value: unknown): value is Permission {
	return value === 'manage_administration'
		|| permissionGroups.some((group) => group.permissions.some((permission) => permission.key === value));
}

export const defaultRolePermissions: Record<AccessRole, readonly Permission[]> = {
	admin: [...businessPermissions, 'manage_administration'],
	staff: businessPermissions,
	readonly: []
};

function isAccessRole(role: string): role is AccessRole {
	return role === 'admin' || role === 'staff' || role === 'readonly';
}

export function hasPermission(
	role: string | null | undefined,
	permission: Permission,
	effectivePermissions?: readonly Permission[]
): boolean {
	if (effectivePermissions) return effectivePermissions.includes(permission);
	return role !== undefined && role !== null && isAccessRole(role)
		&& defaultRolePermissions[role].includes(permission);
}

export function canViewBusinessView(
	role: string | null | undefined,
	view: BusinessView,
	effectivePermissions?: readonly Permission[]
): boolean {
	const permission: Record<BusinessView, Permission> = {
		properties: 'view_properties',
		units: 'view_units',
		tenants: 'view_tenants',
		owners: 'view_owners',
		payments: 'view_payments'
	};
	return hasPermission(role, permission[view], effectivePermissions);
}
