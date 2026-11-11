import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig } from 'vite';

const uiLibrary = new URL('../../packages/ui/src/lib/', import.meta.url).pathname.replace(
	/^\/([A-Za-z]:)/,
	'$1'
);

export default defineConfig({
	resolve: {
		alias: [
			{ find: /^@aqardesk\/ui$/, replacement: `${uiLibrary}index.ts` },
			{ find: /^@aqardesk\/ui\//, replacement: uiLibrary }
		]
	},
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			adapter: adapter({ fallback: 'index.html' })
		}),
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide'
		})
	]
});
