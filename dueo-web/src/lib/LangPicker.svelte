<script lang="ts">
	import { Languages, Check } from '@lucide/svelte';
	import { fly, fade } from 'svelte/transition';
	import { i18n, LANGS } from './i18n.svelte';
	import { updateSettings } from './api';

	let open = $state(false);

	function pick(code: string) {
		i18n.set(code);
		open = false;
		// Sincroniza el idioma con el servidor para que los RECORDATORIOS lleguen en
		// este idioma. Best-effort: si no hay sesión (login) falla en silencio.
		updateSettings({ lang: code }).catch(() => {});
	}
</script>

<div class="wrap">
	<button
		class="trigger"
		onclick={() => (open = !open)}
		aria-label={i18n.t('topbar.lang')}
		title={i18n.t('topbar.lang')}
		aria-expanded={open}
	>
		<Languages size={16} />
		<span>{i18n.lang.toUpperCase()}</span>
	</button>

	{#if open}
		<button
			type="button"
			class="backdrop"
			aria-label={i18n.t('common.close')}
			onclick={() => (open = false)}
			transition:fade={{ duration: 120 }}
		></button>
		<div class="menu" transition:fly={{ y: -6, duration: 160 }}>
			{#each LANGS as l (l.code)}
				<button class="opt" class:on={i18n.lang === l.code} onclick={() => pick(l.code)}>
					<span class="code">{l.code.toUpperCase()}</span>
					<span class="name">{l.label}</span>
					{#if i18n.lang === l.code}<Check size={15} />{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.wrap {
		position: relative;
		display: flex;
	}
	.trigger {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 40px;
		padding: 0 0.7rem;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
		font-size: 0.78rem;
		font-weight: 650;
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s,
			transform 0.15s;
	}
	.trigger:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}
	.trigger:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 2px;
	}
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 40;
		background: transparent;
		border: none;
		padding: 0;
		cursor: default;
	}
	.menu {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		z-index: 41;
		min-width: 160px;
		padding: 5px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 12px;
		box-shadow: 0 14px 34px -14px rgba(0, 0, 0, 0.55);
	}
	.opt {
		display: flex;
		align-items: center;
		gap: 0.55rem;
		padding: 8px 10px;
		border: none;
		background: transparent;
		border-radius: 8px;
		color: var(--text);
		font-size: 0.85rem;
		text-align: left;
		cursor: pointer;
		transition: background 0.12s;
	}
	.opt:hover {
		background: color-mix(in srgb, var(--text) 8%, transparent);
	}
	.opt.on {
		color: var(--brand);
	}
	.opt .code {
		font-size: 0.7rem;
		font-weight: 700;
		color: var(--text-muted);
		width: 22px;
	}
	.opt.on .code {
		color: var(--brand);
	}
	.opt .name {
		flex: 1;
	}
</style>
