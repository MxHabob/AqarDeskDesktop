export const roleQueryKeys = {
	all: (organizationId: string) => ['role-permissions', organizationId] as const,
	pages: (organizationId: string) => ['role-permissions', organizationId, 'pages'] as const
};
