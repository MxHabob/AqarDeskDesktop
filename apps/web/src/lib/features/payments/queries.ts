export const paymentQueryKeys = {
	all: (organizationId: string) => ['payments', organizationId] as const
};
