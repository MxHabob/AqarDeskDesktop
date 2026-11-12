export const paymentQueryKeys = {
	all: (organizationId: string) => ['payments', organizationId] as const,
	pages: (organizationId: string, propertyId?: string) =>
		['payments', organizationId, 'pages', propertyId ?? null] as const
};
