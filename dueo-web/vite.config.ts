// adapter-static: exports the front as a static site (SPA with an index.html
// fallback) that the Rust binary embeds via rust-embed and serves from Axum.
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// index.html fallback → SPA mode: the client resolves any route
			// (no prerendering; ssr is already off in +layout.ts). Output in build/.
			adapter: adapter({ fallback: 'index.html' })
		})
	],
	// Dev: forward /api to the Rust backend (:3000). Same origin → the cookie works.
	server: {
		proxy: {
			'/api': 'http://127.0.0.1:3000'
		}
	}
});
