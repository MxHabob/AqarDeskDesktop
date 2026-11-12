import type { Permission } from '#lib/features/auth/permissions';

export type User = {
	id: string;
	organization_id: string;
	name: string;
	email: string;
	role: string;
	role_id: string;
	role_name: string | null;
	permissions: Permission[];
	is_active: boolean;
	created_at: string;
};
