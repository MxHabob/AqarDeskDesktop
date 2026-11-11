import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
	input: '../../backend/api/openapi.json',
	output: 'src/generated',
	plugins: ['@hey-api/client-fetch']
});
