export const ownerQueryKeys = {
	all: (organizationId: string) => ['owners', organizationId] as const
};
