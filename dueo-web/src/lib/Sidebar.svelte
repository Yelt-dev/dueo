<script lang="ts">
	import { page } from '$app/state';
	import { LayoutDashboard, LineChart, Tags, Settings } from '@lucide/svelte';
	import RingMark from './RingMark.svelte';
	import { i18n } from './i18n.svelte';

	const items = $derived([
		{ icon: LayoutDashboard, label: i18n.t('nav.dashboard'), href: '/' },
		{ icon: LineChart, label: i18n.t('nav.insights'), href: '/insights' },
		{ icon: Tags, label: i18n.t('nav.categories'), href: '/categorias' },
		{ icon: Settings, label: i18n.t('nav.settings'), href: '/ajustes' }
	]);
</script>

<nav class="rail acrylic" aria-label={i18n.t('nav.aria')}>
	<a class="brand" href="/" aria-label={i18n.t('topbar.home')}>
		<RingMark size={26} stroke={4} gap={0} rot={40} />
	</a>

	<ul>
		{#each items as it (it.href)}
			<li>
				<a
					class="item"
					class:active={page.url.pathname === it.href}
					href={it.href}
					aria-label={it.label}
				>
					<it.icon size={20} />
					<span class="tip">{it.label}</span>
				</a>
			</li>
		{/each}
	</ul>
</nav>

<style>
	.rail {
		position: fixed;
		left: 0;
		top: 0;
		bottom: 0;
		width: 64px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1.25rem;
		padding: 1rem 0;
		/* solo borde derecho (la clase .acrylic pone borde completo) */
		border: none;
		border-right: 1px solid var(--border);
		border-radius: 0;
		z-index: 20;
	}
	.brand {
		display: grid;
		place-items: center;
		line-height: 0;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.item {
		position: relative;
		display: grid;
		place-items: center;
		width: 44px;
		height: 44px;
		border-radius: 12px;
		color: var(--text-muted);
		text-decoration: none;
		transition:
			color 0.15s,
			background 0.15s;
	}
	.item:hover {
		color: var(--text);
		background: var(--surface-2);
	}
	.item.active {
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
	}
	.tip {
		position: absolute;
		left: calc(100% + 10px);
		top: 50%;
		transform: translateY(-50%);
		padding: 4px 9px;
		border-radius: 8px;
		background: var(--surface-2);
		color: var(--text);
		border: 1px solid var(--border);
		font-size: 0.78rem;
		white-space: nowrap;
		opacity: 0;
		pointer-events: none;
		transition: opacity 0.12s;
		z-index: 30;
	}
	.item:hover .tip {
		opacity: 1;
	}
</style>
