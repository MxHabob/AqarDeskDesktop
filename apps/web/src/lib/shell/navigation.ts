export type ViewKey =
	| 'overview'
	| 'properties'
	| 'units'
	| 'tenants'
	| 'contracts'
	| 'payments'
	| 'users'
	| 'settings'
	| 'databases'
	| 'account';

export function getViewPath(organizationId: string, view: ViewKey) {
	if (view === 'overview') return `/${organizationId}`;
	if (view === 'users') return `/${organizationId}/settings/users`;
	if (view === 'databases') return `/${organizationId}/settings/databases`;
	if (view === 'account') return `/${organizationId}/account`;
	return `/${organizationId}/${view}`;
}
