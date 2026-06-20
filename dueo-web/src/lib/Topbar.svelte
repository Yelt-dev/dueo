<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { LogOut, Menu, X } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import Logo from './Logo.svelte';
	import ThemeToggle from './ThemeToggle.svelte';
	import LangPicker from './LangPicker.svelte';
	import NotificationsBell from './NotificationsBell.svelte';
	import { me, logout, updateSettings } from './api';
	import { i18n } from './i18n.svelte';

	// Chrome global: presente en todas las pantallas (menos /login). Así la
	// campanita, el tema, el usuario y el logout están SIEMPRE accesibles.
	let user = $state<{ username: string; role: string } | null>(null);

	// En móvil los controles no se amontonan: se colapsan tras un botón y se
	// despliegan en abanico (ver CSS). En desktop el botón está oculto y la fila
	// se muestra entera, así que este estado no afecta.
	let menuOpen = $state(false);

	onMount(async () => {
		const m = await me();
		if (m.ok) {
			const u = await m.json();
			user = u;
			// Si el idioma del servidor (para los recordatorios) no coincide con el de
			// la UI, lo sincronizamos. Cubre el primer login en un dispositivo nuevo.
			if (u.lang && u.lang !== i18n.lang) updateSettings({ lang: i18n.lang }).catch(() => {});
		}
	});

	async function doLogout() {
		await logout();
		goto('/login');
	}
</script>

<header class="topbar acrylic">
	<div class="inner">
		<a class="brand" href="/" aria-label={i18n.t('topbar.home')}><Logo size={34} /></a>
		<div class="actions">
			{#if user}<span class="user">{user.username}</span>{/if}

			<!-- Cluster de controles. En móvil cada uno va en un .slot que se anima
			     (opacidad + desplazamiento) para el efecto de abanico. Un único
			     NotificationsBell (mantiene una sola conexión SSE). -->
			<div class="cluster" class:open={menuOpen}>
				<div class="slot"><NotificationsBell /></div>
				<div class="slot"><LangPicker /></div>
				<div class="slot"><ThemeToggle /></div>
				<div class="slot">
					<button
						class="icon"
						onclick={doLogout}
						aria-label={i18n.t('topbar.logout')}
						title={i18n.t('topbar.logout')}
					>
						<LogOut size={18} />
					</button>
				</div>
			</div>

			<!-- Disparador del abanico (solo visible en móvil). -->
			<button
				class="fab"
				class:open={menuOpen}
				onclick={() => (menuOpen = !menuOpen)}
				aria-label={i18n.t('topbar.menu')}
				aria-expanded={menuOpen}
			>
				{#if menuOpen}<X size={18} />{:else}<Menu size={18} />{/if}
			</button>
		</div>
	</div>

	{#if menuOpen}
		<button
			type="button"
			class="fan-backdrop"
			aria-label={i18n.t('common.close')}
			onclick={() => (menuOpen = false)}
			transition:fade={{ duration: 120 }}
		></button>
	{/if}
</header>

<style>
	.topbar {
		position: sticky;
		top: 0;
		z-index: 30;
		/* la clase .acrylic da fondo+blur; solo queremos borde inferior */
		border: none;
		border-bottom: 1px solid var(--border);
		border-radius: 0;
	}
	.inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		width: 100%;
		max-width: var(--page-max);
		margin-inline: auto;
		padding: 0.6rem var(--page-pad-x);
	}
	.brand {
		display: inline-flex;
		align-items: center;
		line-height: 0;
		text-decoration: none;
	}
	.actions {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0.6rem;
	}
	.cluster {
		display: flex;
		align-items: center;
		gap: 0.6rem;
	}
	/* En desktop el slot es transparente al layout: la fila se ve como siempre. */
	.slot {
		display: contents;
	}
	.user {
		font-size: 0.85rem;
		color: var(--text-2);
		margin-right: 0.2rem;
	}
	.icon {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s,
			transform 0.15s;
	}
	.icon:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}

	/* Disparador del abanico: oculto en desktop. */
	.fab {
		display: none;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s,
			background 0.2s,
			transform 0.2s;
	}

	.fan-backdrop {
		position: fixed;
		inset: 0;
		z-index: 1;
		background: transparent;
		border: none;
		padding: 0;
		cursor: default;
	}

	/* === Móvil: los controles se colapsan y se despliegan en abanico === */
	@media (max-width: 640px) {
		.inner {
			gap: 0.5rem;
			padding: 0.55rem var(--page-pad-x);
		}
		.user {
			display: none;
		}
		/* la fila de controles queda por encima del backdrop y es clicable */
		.actions {
			z-index: 2;
		}

		.fab {
			display: grid;
			position: relative;
			z-index: 1;
		}
		.fab:active {
			transform: scale(0.92);
		}
		.fab.open {
			color: var(--text);
			border-color: var(--border-strong);
			background: color-mix(in srgb, var(--brand) 16%, var(--surface-2));
		}

		/* El cluster cae en columna bajo el disparador, sin empujar el layout.
		   Sin transform en el contenedor (rompería el position:fixed de los
		   popovers internos): se ancla con top/right. */
		.cluster {
			position: absolute;
			top: calc(100% + 0.5rem);
			right: 0;
			flex-direction: column;
			align-items: flex-end;
			gap: 0.4rem;
		}
		.slot {
			display: block;
			opacity: 0;
			transform: translateY(-10px) scale(0.82);
			pointer-events: none;
			transition:
				opacity 0.18s ease,
				transform 0.24s cubic-bezier(0.2, 0.8, 0.2, 1);
		}
		.cluster.open .slot {
			opacity: 1;
			transform: none;
			pointer-events: auto;
		}
		/* Stagger: de arriba (más cercano al disparador) hacia abajo. */
		.cluster.open .slot:nth-child(1) {
			transition-delay: 0s;
		}
		.cluster.open .slot:nth-child(2) {
			transition-delay: 0.045s;
		}
		.cluster.open .slot:nth-child(3) {
			transition-delay: 0.09s;
		}
		.cluster.open .slot:nth-child(4) {
			transition-delay: 0.135s;
		}
	}
</style>
