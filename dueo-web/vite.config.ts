// adapter-static: exporta el front como sitio estático (SPA con fallback
// index.html) que el binario Rust embebe con rust-embed y sirve desde Axum.
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

			// fallback index.html → modo SPA: cualquier ruta la resuelve el cliente
			// (no prerenderizamos; ssr ya está off en +layout.ts). Salida en build/.
			adapter: adapter({ fallback: 'index.html' })
		})
	],
	// Dev: reenvía /api al backend Rust (:3000). Mismo origen → la cookie funciona.
	server: {
		proxy: {
			'/api': 'http://127.0.0.1:3000'
		}
	}
});
