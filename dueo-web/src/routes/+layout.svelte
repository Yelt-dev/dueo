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

	// Chrome (rail + topbar) is hidden on the login page.
	const showChrome = $derived(page.url.pathname !== '/login');

	// Per-section document title (the tab showed the URL when no title was set).
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

	// Note: the brand catalog (Simple Icons, ~5MB) is NOT loaded here; it loads
	// on demand when searching for an icon (see modal) or if subs use an explicit brand.
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
