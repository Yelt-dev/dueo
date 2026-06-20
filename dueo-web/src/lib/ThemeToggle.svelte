<script lang="ts">
	import { Sun, Moon } from '@lucide/svelte';
	import { theme } from './theme.svelte';
	import { i18n } from './i18n.svelte';
</script>

<button
	onclick={() => theme.toggle()}
	aria-label={i18n.t('topbar.theme')}
	title={i18n.t('topbar.theme')}
>
	{#if theme.mode === 'dark'}
		<Sun size={18} />
	{:else}
		<Moon size={18} />
	{/if}
</button>

<style>
	button {
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
	button:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}
	button:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 2px;
	}
</style>
