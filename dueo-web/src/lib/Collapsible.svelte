<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { ChevronDown } from '@lucide/svelte';
	import { slide } from 'svelte/transition';

	// Tarjeta colapsable para secciones largas (p. ej. /ajustes). El cuerpo se pasa
	// como children y se renderiza en el scope del padre (sus estilos siguen aplicando).
	let {
		icon: Icon,
		title,
		desc = '',
		open = false,
		ontoggle,
		children
	}: {
		icon: Component;
		title: string;
		desc?: string;
		open?: boolean;
		ontoggle?: () => void;
		children: Snippet;
	} = $props();
</script>

<section class="card" class:open>
	<button type="button" class="head" onclick={ontoggle} aria-expanded={open}>
		<span class="lead"><Icon size={20} /></span>
		<div class="meta">
			<h2>{title}</h2>
			{#if desc}<p>{desc}</p>{/if}
		</div>
		<span class="chev" class:rot={open}><ChevronDown size={18} /></span>
	</button>

	{#if open}
		<div class="body" transition:slide={{ duration: 200 }}>
			{@render children()}
		</div>
	{/if}
</section>

<style>
	.card {
		margin-bottom: var(--gap-list);
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg, 16px);
		overflow: hidden;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 0.7rem;
		width: 100%;
		padding: 1.2rem 1.4rem;
		border: none;
		background: transparent;
		color: var(--text);
		text-align: left;
		cursor: pointer;
		transition: background 0.15s;
	}
	.head:hover {
		background: color-mix(in srgb, var(--text) 4%, transparent);
	}
	.lead {
		display: grid;
		place-items: center;
		width: 38px;
		height: 38px;
		flex: none;
		border-radius: 11px;
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 26%, transparent);
	}
	.meta {
		flex: 1;
		min-width: 0;
	}
	.meta h2 {
		margin: 0 0 0.15rem;
		font-size: 1rem;
	}
	.meta p {
		margin: 0;
		font-size: 0.82rem;
		color: var(--text-muted);
		line-height: 1.4;
	}
	.chev {
		display: grid;
		place-items: center;
		color: var(--text-muted);
		transition:
			transform 0.2s ease,
			color 0.15s;
	}
	.chev.rot {
		transform: rotate(180deg);
		color: var(--text-2);
	}
	/* el cuerpo: mismo gap/padding que tenían las cards de ajustes.
	   Un pequeño top da aire entre la cabecera y el contenido. */
	.body {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding: 0.5rem 1.4rem 1.3rem;
	}
</style>
