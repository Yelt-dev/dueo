<script lang="ts">
	import { Sparkles, Search } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import { searchIcons, brandsReady, DUEO_COLORS } from './icons';
	import { ensureBrands } from './brandcat.svelte';
	import { i18n } from './i18n.svelte';

	// icon/color = null means automatic (brand-by-name / default tone).
	let {
		icon = $bindable(null),
		color = $bindable(null)
	}: { icon?: string | null; color?: string | null } = $props();

	let iconQuery = $state('');
	// Reactive to the lazy catalogue: generic icons show instantly; brands appear
	// once the user focuses the search (loaded on demand).
	const filteredIcons = $derived(searchIcons(iconQuery));
</script>

<div class="vis">
	<div class="vishead">
		<span class="rlabel">{i18n.t('modal.iconColor')}</span>
		<button
			type="button"
			class="auto"
			class:on={icon === null && color === null}
			onclick={() => {
				icon = null;
				color = null;
			}}
		>
			<Sparkles size={13} />
			{i18n.t('modal.auto')}
		</button>
	</div>

	<div class="searchrow">
		<Search size={14} />
		<input bind:value={iconQuery} onfocus={ensureBrands} placeholder={i18n.t('modal.iconSearch')} />
		{#if iconQuery && !brandsReady()}<span class="loadingdot" title={i18n.t('modal.loadingBrands')}
			></span>{/if}
	</div>

	<div class="iconscroll">
		{#each filteredIcons as ic (ic.id)}
			<button
				type="button"
				class="iconbtn"
				class:sel={icon === ic.id}
				title={ic.label}
				aria-label={ic.label}
				onclick={() => (icon = ic.id)}
			>
				<Icon def={ic} size={18} />
			</button>
		{/each}
	</div>

	<div class="colorrow">
		{#each DUEO_COLORS as c (c)}
			<button
				type="button"
				class="dot"
				class:on={color === c}
				style="--d:{c}"
				aria-label={i18n.t('modal.color')}
				onclick={() => (color = c)}
			></button>
		{/each}
		<label class="custom" title={i18n.t('modal.customColor')}>
			<input type="color" oninput={(e) => (color = e.currentTarget.value)} />
			<span style="--d:{color ?? 'var(--brand)'}"></span>
		</label>
	</div>
</div>

<style>
	.vis {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding: 0.7rem 0.8rem;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: var(--surface-2);
	}
	.vishead {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.rlabel {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text);
	}
	.auto {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		border-radius: 999px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		font-size: 0.74rem;
		font-weight: 600;
		cursor: pointer;
	}
	.auto.on {
		color: white;
		border-color: transparent;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.searchrow {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		padding: 0 0.6rem;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--surface);
		color: var(--text-muted);
	}
	.searchrow input {
		border: none;
		background: transparent;
		padding: 0.45rem 0;
		flex: 1;
		color: var(--text);
		font-size: 0.86rem;
	}
	.searchrow input:focus-visible {
		outline: none;
	}
	.loadingdot {
		width: 12px;
		height: 12px;
		border-radius: 999px;
		border: 2px solid var(--border);
		border-top-color: var(--brand);
		animation: spin 0.7s linear infinite;
		flex: none;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.iconscroll {
		display: grid;
		grid-template-columns: repeat(8, 1fr);
		gap: 5px;
		max-height: 132px;
		overflow-y: auto;
		padding: 2px;
	}
	.iconbtn {
		display: grid;
		place-items: center;
		aspect-ratio: 1;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		cursor: pointer;
		transition:
			color 0.12s,
			border-color 0.12s,
			transform 0.12s;
	}
	.iconbtn:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}
	.iconbtn.sel {
		color: var(--brand);
		border-color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
	}
	.colorrow {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 7px;
	}
	.dot {
		width: 22px;
		height: 22px;
		border-radius: 999px;
		border: 2px solid transparent;
		background: var(--d);
		cursor: pointer;
		padding: 0;
		transition: transform 0.12s;
	}
	.dot:hover {
		transform: scale(1.12);
	}
	.dot.on {
		border-color: var(--text);
	}
	.custom {
		position: relative;
		width: 22px;
		height: 22px;
		cursor: pointer;
	}
	.custom input {
		position: absolute;
		inset: 0;
		opacity: 0;
		cursor: pointer;
	}
	.custom span {
		display: block;
		width: 22px;
		height: 22px;
		border-radius: 999px;
		background: conic-gradient(from 0deg, #ef6b3d, #ef4da3, #a64def, #4d74ef, #ef6b3d);
		border: 2px solid var(--border-strong);
	}
</style>
