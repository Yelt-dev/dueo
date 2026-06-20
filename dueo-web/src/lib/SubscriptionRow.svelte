<script lang="ts">
	import {
		CircleCheck,
		Pause,
		TriangleAlert,
		Calendar,
		CreditCard,
		Wallet,
		Ellipsis,
		RefreshCw,
		Pencil,
		Trash2
	} from '@lucide/svelte';
	import type { Component } from 'svelte';
	import { timeColor } from './format';
	import { i18n, daysLabel } from './i18n.svelte';
	import Icon from './Icon.svelte';
	import type { IconDef } from './icons';

	let {
		name,
		icon: Fallback,
		iconDef = null,
		catLabel,
		chipColor = 'var(--brand)',
		amountLabel,
		cycle,
		days,
		progress,
		status = 'active',
		paymentMode = 'manual',
		brand = null,
		canRenew = true,
		domId = undefined,
		highlighted = false,
		onrenew,
		onedit,
		ondelete
	}: {
		name: string;
		icon: Component;
		iconDef?: IconDef | null;
		catLabel: string;
		chipColor?: string;
		amountLabel: string;
		cycle: string;
		days: number;
		progress: number;
		status?: string;
		paymentMode?: string;
		brand?: { path: string; color: string } | null;
		canRenew?: boolean;
		domId?: string;
		highlighted?: boolean;
		onrenew?: () => void;
		onedit?: () => void;
		ondelete?: () => void;
	} = $props();

	const c = $derived(timeColor(progress));
	const fill = $derived(Math.round(progress * 100));

	let menuOpen = $state(false);
	let confirmDelete = $state(false);
	function closeMenu() {
		menuOpen = false;
		confirmDelete = false;
	}

	const badge = $derived(
		status === 'paused'
			? { label: i18n.t('row.paused'), icon: Pause, color: 'var(--text-muted)' }
			: status === 'expired'
				? { label: i18n.t('row.expired'), icon: TriangleAlert, color: 'var(--danger)' }
				: { label: i18n.t('row.active'), icon: CircleCheck, color: 'var(--ok)' }
	);
</script>

<article class="row" class:highlighted id={domId} style="--c:{c}">
	<div class="left">
		<span class="chip" style="--cc:{chipColor}">
			{#if iconDef}
				<Icon def={iconDef} size={18} />
			{:else if brand}
				<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" aria-hidden="true">
					<path d={brand.path} />
				</svg>
			{:else}
				<Fallback size={16} />
			{/if}
		</span>
		<div class="names">
			<span class="name">{name}</span>
			<span class="cat">{catLabel}</span>
		</div>
	</div>

	<div class="mid">
		<div class="bar">
			<div class="fillwrap">
				<div class="fill" style="width:{fill}%"></div>
			</div>
		</div>
		<div class="under">
			<span class="badge" style="--bc:{badge.color}">
				<badge.icon size={12} />
				{badge.label}
			</span>
			<span class="due">
				<Calendar size={12} />
				{daysLabel(days)}
			</span>
		</div>
	</div>

	<div class="right">
		<span class="amount tnum">{amountLabel}</span>
		<span class="cycle">
			{#if paymentMode === 'auto'}<CreditCard size={12} />{:else}<Wallet size={12} />{/if}
			/ {cycle}
		</span>
	</div>

	<div class="menu">
		<button
			class="menubtn"
			aria-label={i18n.t('row.actions')}
			onclick={() => {
				menuOpen = !menuOpen;
				confirmDelete = false;
			}}
		>
			<Ellipsis size={18} />
		</button>
		{#if menuOpen}
			<button
				type="button"
				class="backdrop"
				aria-label={i18n.t('row.closeMenu')}
				onclick={closeMenu}
			></button>
			<div class="dropdown">
				{#if canRenew}
					<button
						onclick={() => {
							closeMenu();
							onrenew?.();
						}}><RefreshCw size={15} /> {i18n.t('row.renew')}</button
					>
				{/if}
				<button
					onclick={() => {
						closeMenu();
						onedit?.();
					}}><Pencil size={15} /> {i18n.t('common.edit')}</button
				>
				{#if !confirmDelete}
					<button class="danger" onclick={() => (confirmDelete = true)}>
						<Trash2 size={15} />
						{i18n.t('common.delete')}
					</button>
				{:else}
					<button
						class="danger"
						onclick={() => {
							closeMenu();
							ondelete?.();
						}}><Trash2 size={15} /> {i18n.t('common.confirmDelete')}</button
					>
				{/if}
			</div>
		{/if}
	</div>
</article>

<style>
	.row {
		display: grid;
		grid-template-columns: minmax(160px, 1.05fr) 1.9fr minmax(96px, 0.6fr) 34px;
		align-items: center;
		gap: 1.25rem;
		padding: 1rem 1.5rem 1rem 2.2rem;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg, 16px);
		transition:
			transform 0.16s ease,
			border-color 0.16s ease,
			box-shadow 0.16s ease;
	}
	.row:hover {
		transform: translateY(-2px);
		border-color: color-mix(in srgb, var(--c) 45%, var(--border));
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--c) 25%, transparent),
			0 10px 30px -12px color-mix(in srgb, var(--c) 40%, transparent);
	}
	/* resalte al llegar desde el Horizonte (clic en el marcador) */
	.row.highlighted {
		border-color: var(--brand);
		animation: rowflash 1.8s ease-out;
	}
	@keyframes rowflash {
		0%,
		28% {
			box-shadow:
				0 0 0 2px color-mix(in srgb, var(--brand) 60%, transparent),
				0 0 26px -4px color-mix(in srgb, var(--brand) 55%, transparent);
		}
		100% {
			box-shadow: 0 0 0 0 transparent;
		}
	}

	.left {
		display: flex;
		align-items: center;
		gap: 1rem;
		min-width: 0;
	}
	.chip {
		display: grid;
		place-items: center;
		width: 38px;
		height: 38px;
		border-radius: 11px;
		color: var(--cc);
		background: color-mix(in srgb, var(--cc) 16%, transparent);
		border: 1px solid color-mix(in srgb, var(--cc) 30%, transparent);
		flex: none;
	}
	.names {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.name {
		font-weight: 650;
		font-size: 0.95rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	/* en hover, el nombre completo hace wrap a varias líneas (no se trunca) */
	.row:hover .name {
		white-space: normal;
		overflow: visible;
		word-break: break-word;
		line-height: 1.2;
	}
	.cat {
		font-size: 0.78rem;
		color: var(--text-muted);
	}

	/* Lifebar */
	.bar {
		position: relative;
		height: 12px;
	}
	.fillwrap {
		position: absolute;
		inset: 0;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text-muted) 18%, transparent);
		overflow: hidden;
		border: 1px solid var(--border);
	}
	.fill {
		height: 100%;
		border-radius: 999px;
		background: linear-gradient(90deg, color-mix(in srgb, var(--brand) 55%, var(--c)), var(--c));
		box-shadow: 0 0 12px -2px var(--c);
		animation: grow 0.9s cubic-bezier(0.2, 0.8, 0.2, 1) both;
	}
	@keyframes grow {
		from {
			width: 0 !important;
		}
	}
	.under {
		display: flex;
		align-items: center;
		gap: 0.7rem;
		margin-top: 0.55rem;
	}
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.72rem;
		font-weight: 600;
		color: var(--bc);
		background: color-mix(in srgb, var(--bc) 14%, transparent);
		padding: 2px 8px;
		border-radius: 999px;
	}
	.due {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.76rem;
		color: var(--text-2);
	}

	.right {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
	}
	.amount {
		font-weight: 750;
		font-size: 1rem;
	}
	.cycle {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 0.76rem;
		color: var(--text-muted);
	}

	/* menú de acciones */
	.menu {
		position: relative;
		display: flex;
		justify-content: center;
	}
	.menubtn {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 9px;
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color 0.15s,
			background 0.15s,
			border-color 0.15s;
	}
	.menubtn:hover {
		color: var(--text);
		background: var(--surface-2);
		border-color: var(--border);
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
	.dropdown {
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		z-index: 41;
		min-width: 170px;
		padding: 5px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: 12px;
		box-shadow: 0 14px 34px -14px rgba(0, 0, 0, 0.55);
	}
	.dropdown button {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border: none;
		background: transparent;
		border-radius: 8px;
		color: var(--text);
		font-size: 0.85rem;
		font-weight: 550;
		text-align: left;
		cursor: pointer;
		transition: background 0.12s;
	}
	.dropdown button:hover {
		background: color-mix(in srgb, var(--text) 8%, transparent);
	}
	.dropdown button.danger {
		color: var(--danger);
	}
	.dropdown button.danger:hover {
		background: color-mix(in srgb, var(--danger) 14%, transparent);
	}

	/* Móvil: el grid de 4 columnas no cabe → reorganizamos en filas.
	   Arriba: icono+nombre · monto · menú. Debajo: la lifebar a todo el ancho. */
	@media (max-width: 640px) {
		.row {
			grid-template-columns: 1fr auto auto;
			grid-template-areas:
				'left right menu'
				'mid mid mid';
			gap: 0.7rem 0.75rem;
			padding: 0.95rem 1.1rem;
		}
		.left {
			grid-area: left;
		}
		.right {
			grid-area: right;
		}
		.mid {
			grid-area: mid;
		}
		.menu {
			grid-area: menu;
		}
		/* en hover el nombre ya no necesita truncado raro en móvil */
		.left {
			gap: 0.7rem;
		}
	}
</style>
