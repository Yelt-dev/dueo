import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';
import ts from 'typescript-eslint';

export default ts.config(
	// Generado por build / SvelteKit, no se lintea.
	{
		ignores: ['build/', '.svelte-kit/', 'dist/']
	},
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs.recommended,
	// prettier debe ir al final: apaga las reglas de estilo que chocan con el formateo.
	prettier,
	...svelte.configs.prettier,
	{
		languageOptions: {
			globals: { ...globals.browser, ...globals.node }
		},
		rules: {
			// TS resuelve los símbolos; no-undef da falsos positivos (p.ej. runes $state).
			'no-undef': 'off',
			// Permite descartar bindings con `_` (p.ej. `{#each n as _, i}`).
			'@typescript-eslint/no-unused-vars': [
				'error',
				{ argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' }
			],
			// Dueo es una SPA con rutas string fijas; resolve() solo añadiría ruido.
			'svelte/no-navigation-without-resolve': 'off',
			// Falsos positivos: los Map/Date marcados son cálculos locales dentro de
			// $derived.by(), no estado reactivo que requiera SvelteMap/SvelteDate.
			'svelte/prefer-svelte-reactivity': 'off'
		}
	},
	{
		// Los archivos .svelte (y módulos de runes .svelte.ts/.js) se parsean con TS.
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				parser: ts.parser
			}
		}
	}
);
