export const roleQueryKeys = {
	all: (organizationId: string) => ['role-permissions', organizationId] as const
};
