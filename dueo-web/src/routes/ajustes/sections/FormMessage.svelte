<script lang="ts">
	import { slide } from 'svelte/transition';

	// Inline result line shared by every settings section: green on success,
	// red on failure, animated in/out.
	let { msg }: { msg: { ok: boolean; text: string } | null } = $props();
</script>

{#if msg}
	<p transition:slide={{ duration: 180 }} class:err={!msg.ok} class:ok={msg.ok}>{msg.text}</p>
{/if}

<style>
	p {
		margin: 0;
		font-size: 0.82rem;
	}
	.err {
		color: var(--danger);
	}
	.ok {
		color: var(--ok);
	}
</style>
