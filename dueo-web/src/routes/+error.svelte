<script lang="ts">
	import { page } from '$app/state';
	import { fly } from 'svelte/transition';
	import { Home, Compass, TriangleAlert } from '@lucide/svelte';
	import { i18n } from '$lib/i18n.svelte';
	import Logo from '$lib/Logo.svelte';

	const isNotFound = $derived(page.status === 404);
	const title = $derived(isNotFound ? i18n.t('err.notFoundTitle') : i18n.t('err.genericTitle'));
	const text = $derived(isNotFound ? i18n.t('err.notFoundText') : i18n.t('err.genericText'));
</script>

<div class="wrap">
	<div class="card acrylic" in:fly={{ y: 14, duration: 320 }}>
		<Logo size={36} />
		<span class="icon" class:warn={!isNotFound}>
			{#if isNotFound}<Compass size={30} />{:else}<TriangleAlert size={30} />{/if}
		</span>
		<span class="code">{page.status}</span>
		<h1>{title}</h1>
		<p>{text}</p>
		<a class="home" href="/"><Home size={16} /> {i18n.t('err.home')}</a>
	</div>
</div>

<style>
	.wrap {
		min-height: 70dvh;
		display: grid;
		place-items: center;
		padding: 2rem 1rem;
	}
	.card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		text-align: center;
		max-width: 380px;
		padding: 2.2rem 2rem;
		border-radius: var(--radius-xl, 20px);
	}
	.icon {
		display: grid;
		place-items: center;
		width: 60px;
		height: 60px;
		margin-top: 0.5rem;
		border-radius: 16px;
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 26%, transparent);
	}
	.icon.warn {
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 14%, transparent);
		border-color: color-mix(in srgb, var(--warn) 26%, transparent);
	}
	.code {
		margin-top: 0.4rem;
		font-size: 0.8rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}
	h1 {
		margin: 0;
		font-size: 1.25rem;
	}
	p {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.9rem;
		line-height: 1.45;
	}
	.home {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		margin-top: 1rem;
		height: 42px;
		padding: 0 1.2rem;
		border-radius: 12px;
		font-weight: 650;
		font-size: 0.9rem;
		color: white;
		text-decoration: none;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
		transition: transform 0.15s;
	}
	.home:hover {
		transform: translateY(-1px);
	}
</style>
