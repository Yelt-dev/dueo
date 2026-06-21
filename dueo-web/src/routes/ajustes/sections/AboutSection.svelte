<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { ExternalLink } from '@lucide/svelte';
	import { getVersion, getHealth, type VersionInfo } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';

	let versionInfo = $state<VersionInfo | null>(null);
	let backendOk = $state<boolean | null>(null);

	onMount(async () => {
		const v = await getVersion();
		if (v.ok && v.data) versionInfo = v.data;
		try {
			const h = await getHealth();
			backendOk = h.ok;
		} catch {
			backendOk = false;
		}
	});
</script>

<div class="about">
	<div class="arow">
		<span class="muted">{i18n.t('set.version')}</span>
		{#if versionInfo}
			<span transition:fade={{ duration: 150 }}>{versionInfo.name} {versionInfo.version}</span>
		{:else}
			<div class="skeleton" style="width:90px;height:1rem"></div>
		{/if}
	</div>
	<div class="arow">
		<span class="muted">{i18n.t('set.backend')}</span>
		{#if backendOk === null}
			<div class="skeleton" style="width:70px;height:1rem"></div>
		{:else}
			<span class="status" class:up={backendOk} transition:fade={{ duration: 150 }}>
				<span class="dot"></span>{backendOk ? i18n.t('set.online') : i18n.t('set.offline')}
			</span>
		{/if}
	</div>
	<div class="arow">
		<span class="muted">{i18n.t('set.project')}</span>
		<a
			class="link"
			href="https://github.com/Yelt-dev/dueo"
			target="_blank"
			rel="noreferrer noopener"
		>
			{i18n.t('set.repo')}
			<ExternalLink size={13} />
		</a>
	</div>
</div>

<style>
	.about {
		display: flex;
		flex-direction: column;
		gap: 0.55rem;
	}
	.arow {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 0.88rem;
		color: var(--text);
	}
	.status {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--danger);
	}
	.status.up {
		color: var(--ok);
	}
	.status .dot {
		width: 8px;
		height: 8px;
		border-radius: 999px;
		background: currentColor;
	}
	.link {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		color: var(--brand);
		text-decoration: none;
		font-size: 0.86rem;
	}
	.link:hover {
		text-decoration: underline;
	}
</style>
