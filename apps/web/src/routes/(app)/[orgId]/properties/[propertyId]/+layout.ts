import type { LayoutLoad } from './$types';

export const load: LayoutLoad = ({ params }) => ({
	organizationId: params.orgId,
	propertyId: params.propertyId
});
