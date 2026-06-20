<script lang="ts">
	import { iconById, type IconDef } from './icons';

	let {
		id = null,
		def = null,
		size = 18
	}: { id?: string | null; def?: IconDef | null; size?: number } = $props();

	const resolved = $derived(def ?? iconById(id));
</script>

{#if resolved}
	{#if resolved.kind === 'lu'}
		{@const Comp = resolved.comp}
		<Comp {size} />
	{:else}
		<svg viewBox="0 0 24 24" width={size} height={size} fill="currentColor" aria-hidden="true">
			<path d={resolved.path} />
		</svg>
	{/if}
{/if}
