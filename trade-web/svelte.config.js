import adapter from '@sveltejs/adapter-node';
import preprocess from 'svelte-preprocess';

const production = process.env.NODE_ENV === 'production'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://github.com/sveltejs/svelte-preprocess
	// for more information about preprocessors
	preprocess: preprocess(),

	kit: {
		adapter: adapter({
			out: 'build',
			precompress: false,
			envPrefix: ''
		}),

		// Override http methods in the Todo forms
		methodOverride: {
			allowed: ['PATCH', 'DELETE']
		},
		/** @type {import('vite').UserConfig} */
		vite: {
			build: {
				sourcemap: true
			},
			optimizeDeps: {
				include: ['@carbon/charts'],
				exclude: ['@carbon/telemetry']
			},
			ssr: {
				noExternal: [production && "@carbon/charts"].filter(Boolean)
			}
		} // vite

	}
};

export default config;
