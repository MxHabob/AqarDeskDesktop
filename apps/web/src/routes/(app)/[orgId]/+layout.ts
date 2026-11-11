import type { LayoutLoad } from './$types';

export const load: LayoutLoad = ({ params }) => ({ orgId: params.orgId });
