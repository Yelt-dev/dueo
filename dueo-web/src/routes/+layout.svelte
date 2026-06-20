<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { theme } from '$lib/theme.svelte';
	import { i18n } from '$lib/i18n.svelte';
	import Sidebar from '$lib/Sidebar.svelte';
	import Topbar from '$lib/Topbar.svelte';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	// El chrome (rail + topbar) se oculta en el login.
	const showChrome = $derived(page.url.pathname !== '/login');

	// Título del documento por sección (la pestaña mostraba la URL al no haber title).
	const SECTIONS: Record<string, string> = {
		'/': 'doc.home',
		'/login': 'doc.login',
		'/categorias': 'doc.categories',
		'/insights': 'doc.insights',
		'/ajustes': 'doc.settings'
	};
	const title = $derived(
		SECTIONS[page.url.pathname] ? `${i18n.t(SECTIONS[page.url.pathname])} · Dueo` : 'Dueo'
	);

	// Nota: el catálogo de marcas (Simple Icons, ~5MB) NO se carga aquí; se carga
	// bajo demanda al buscar un icono (ver modal) o si hay subs con marca explícita.
	onMount(() => {
		theme.init();
		i18n.init();
	});
</script>

<svelte:head>
	<title>{title}</title>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if showChrome}
	<Sidebar />
{/if}

<div class="content" class:railed={showChrome}>
	{#if showChrome}
		<Topbar />
	{/if}
	{@render children()}
</div>

<style>
	.content.railed {
		padding-left: 64px;
	}
</style>
