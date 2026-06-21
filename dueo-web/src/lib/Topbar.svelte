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

	// Global chrome: present on every screen (except /login), so the bell, theme,
	// user and logout are ALWAYS accessible.
	let user = $state<{ username: string; role: string } | null>(null);

	// On mobile the controls don't pile up: they collapse behind a button and fan
	// out (see CSS). On desktop the button is hidden and the row shows in full, so
	// this state has no effect.
	let menuOpen = $state(false);

	onMount(async () => {
		const m = await me();
		if (m.ok && m.data) {
			const u = m.data;
			user = u;
			// If the server lang (used for reminders) differs from the UI lang, sync it.
			// Covers the first login on a new device.
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

			<!-- Control cluster. On mobile each one sits in a .slot that animates
			     (opacity + translate) for the fan effect. A single NotificationsBell
			     (keeps one SSE connection). -->
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

			<!-- Fan trigger (mobile only). -->
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
		/* .acrylic provides bg+blur; we only want a bottom border */
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
	/* On desktop the slot is layout-transparent: the row looks as usual. */
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

	/* Fan trigger: hidden on desktop. */
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

	/* === Mobile: controls collapse and fan out === */
	@media (max-width: 640px) {
		.inner {
			gap: 0.5rem;
			padding: 0.55rem var(--page-pad-x);
		}
		.user {
			display: none;
		}
		/* Control row sits above the backdrop and stays clickable */
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

		/* The cluster drops into a column under the trigger without pushing layout.
		   No transform on the container (it would break position:fixed of the inner
		   popovers): anchored with top/right instead. */
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
		/* Stagger: from top (closest to the trigger) downward. */
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
