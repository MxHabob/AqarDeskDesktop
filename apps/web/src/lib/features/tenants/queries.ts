export const tenantQueryKeys = {
	all: (organizationId: string) => ['tenants', organizationId] as const
};
