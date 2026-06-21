<script lang="ts">
	import { Languages, Check } from '@lucide/svelte';
	import { i18n, LANGS } from './i18n.svelte';
	import { updateSettings } from './api';
	import Popover from './Popover.svelte';

	let open = $state(false);

	function pick(code: string) {
		i18n.set(code);
		open = false;
		// Sync the lang to the server so REMINDERS arrive in this language.
		// Best-effort: fails silently if there's no session (login).
		updateSettings({ lang: code }).catch(() => {});
	}
</script>

<Popover bind:open style="--pop-min-width: 160px">
	{#snippet trigger({ open, toggle })}
		<button
			class="trigger"
			onclick={toggle}
			aria-label={i18n.t('topbar.lang')}
			title={i18n.t('topbar.lang')}
			aria-haspopup="menu"
			aria-expanded={open}
		>
			<Languages size={16} />
			<span>{i18n.lang.toUpperCase()}</span>
		</button>
	{/snippet}

	{#each LANGS as l (l.code)}
		<button class="opt" class:on={i18n.lang === l.code} onclick={() => pick(l.code)}>
			<span class="code">{l.code.toUpperCase()}</span>
			<span class="name">{l.label}</span>
			{#if i18n.lang === l.code}<Check size={15} />{/if}
		</button>
	{/each}
</Popover>

<style>
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
		width: 100%;
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
