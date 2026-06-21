<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fly, fade } from 'svelte/transition';
	import { i18n } from './i18n.svelte';

	// Anchored dropdown shell: trigger + a panel that opens below it. Owns the
	// behaviour every dropdown shares (click-outside backdrop, Escape, focus
	// return) so call sites only supply content. Chrome is themable via CSS vars
	// (--pop-bg/-width/-min-width/-pad/-radius/-overflow/-shadow) set through `style`.
	let {
		open = $bindable(false),
		align = 'right',
		closeLabel,
		style = '',
		onopen,
		onclose,
		trigger,
		children
	}: {
		open?: boolean;
		align?: 'left' | 'right';
		closeLabel?: string;
		style?: string;
		onopen?: () => void;
		onclose?: () => void;
		trigger: Snippet<[{ open: boolean; toggle: () => void }]>;
		children: Snippet<[{ close: () => void }]>;
	} = $props();

	// Element focused before opening, so we can hand focus back on close (a menu
	// that steals focus and never returns it strands keyboard users).
	let lastFocused: HTMLElement | null = null;

	function toggle() {
		if (open) close();
		else openPop();
	}

	function openPop() {
		lastFocused = document.activeElement as HTMLElement | null;
		open = true;
		onopen?.();
	}

	function close() {
		if (!open) return;
		open = false;
		onclose?.();
		lastFocused?.focus?.();
	}
</script>

<svelte:window onkeydown={(e) => open && e.key === 'Escape' && close()} />

<div class="pop-wrap" {style}>
	{@render trigger({ open, toggle })}

	{#if open}
		<button
			type="button"
			class="pop-backdrop"
			aria-label={closeLabel ?? i18n.t('common.close')}
			onclick={close}
			transition:fade={{ duration: 120 }}
		></button>
		<div class="pop-panel" class:left={align === 'left'} transition:fly={{ y: -6, duration: 160 }}>
			{@render children({ close })}
		</div>
	{/if}
</div>

<style>
	.pop-wrap {
		position: relative;
		display: flex;
	}
	.pop-backdrop {
		position: fixed;
		inset: 0;
		z-index: 40;
		background: transparent;
		border: none;
		padding: 0;
		cursor: default;
	}
	.pop-panel {
		position: absolute;
		top: calc(100% + var(--pop-gap, 8px));
		right: 0;
		z-index: 41;
		display: flex;
		flex-direction: column;
		gap: var(--pop-gap-items, 2px);
		min-width: var(--pop-min-width, 170px);
		width: var(--pop-width, auto);
		max-width: var(--pop-max-width, 86vw);
		padding: var(--pop-pad, 5px);
		background: var(--pop-bg, var(--surface));
		border: 1px solid var(--border);
		border-radius: var(--pop-radius, 12px);
		box-shadow: var(--pop-shadow, 0 14px 34px -14px rgba(0, 0, 0, 0.55));
		overflow: var(--pop-overflow, visible);
	}
	.pop-panel.left {
		right: auto;
		left: 0;
	}
</style>
