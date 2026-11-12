export const tenantQueryKeys = {
	all: (organizationId: string) => ['tenants', organizationId] as const,
	pages: (organizationId: string) => ['tenants', organizationId, 'pages'] as const
};
