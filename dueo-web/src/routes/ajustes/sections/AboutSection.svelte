<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, slide } from 'svelte/transition';
	import { ExternalLink } from '@lucide/svelte';
	import { getVersion, getHealth, getUpdate, type VersionInfo, type UpdateInfo } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';

	let versionInfo = $state<VersionInfo | null>(null);
	let backendOk = $state<boolean | null>(null);
	// null while loading; after that, `checked:false` means we don't show the row.
	let update = $state<UpdateInfo | null>(null);
	let updateLoaded = $state(false);

	// Distribution is the published GHCR image (compose `image:` + `latest`), so
	// the update step is a pull + restart; data lives in the volume, not the image.
	const UPDATE_CMD = 'docker compose pull && docker compose up -d';

	onMount(async () => {
		const v = await getVersion();
		if (v.ok && v.data) versionInfo = v.data;
		try {
			const h = await getHealth();
			backendOk = h.ok;
		} catch {
			backendOk = false;
		}
		const u = await getUpdate();
		if (u.ok && u.data) update = u.data;
		updateLoaded = true;
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
	{#if !updateLoaded}
		<div class="arow">
			<span class="muted">{i18n.t('set.update')}</span>
			<div class="skeleton" style="width:130px;height:1rem"></div>
		</div>
	{:else if update?.checked}
		<div class="arow" transition:fade={{ duration: 150 }}>
			<span class="muted">{i18n.t('set.update')}</span>
			{#if update.update_available}
				<a class="link" href={update.url} target="_blank" rel="noreferrer noopener">
					{i18n.t('set.updateAvail', { version: update.latest ?? '' })}
					<ExternalLink size={13} />
				</a>
			{:else}
				<span class="status up"><span class="dot"></span>{i18n.t('set.updateLatest')}</span>
			{/if}
		</div>
		{#if update.update_available}
			<div class="howto" transition:slide={{ duration: 180 }}>
				<span class="muted">{i18n.t('set.updateCmd')}</span>
				<code>{UPDATE_CMD}</code>
			</div>
		{/if}
	{/if}
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
	.howto {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		font-size: 0.82rem;
	}
	.howto code {
		font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
		font-size: 0.8rem;
		background: var(--surface-2, rgba(127, 127, 127, 0.12));
		border: 1px solid var(--border, rgba(127, 127, 127, 0.2));
		border-radius: 6px;
		padding: 0.45rem 0.6rem;
		overflow-x: auto;
		white-space: nowrap;
		color: var(--text);
	}
</style>
