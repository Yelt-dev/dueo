<script lang="ts">
	import { Download, Upload } from '@lucide/svelte';
	import { exportData, importData } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import { busy } from './busy.svelte';
	import FormMessage from './FormMessage.svelte';

	let msg = $state<{ ok: boolean; text: string } | null>(null);
	let fileInput: HTMLInputElement;
	const importer = busy();

	async function doExport() {
		msg = null;
		const res = await exportData();
		if (!res.ok) return (msg = { ok: false, text: i18n.t('set.exportErr') });
		const blob = new Blob([JSON.stringify(res.data, null, 2)], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		const date = new Date().toISOString().slice(0, 10);
		a.href = url;
		a.download = `dueo-backup-${date}.json`;
		a.click();
		URL.revokeObjectURL(url);
		msg = { ok: true, text: i18n.t('set.exportOk') };
	}

	function onImportFile(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = ''; // allow re-picking the same file
		if (!file) return;
		msg = null;
		importer.run(async () => {
			try {
				const parsed = JSON.parse(await file.text());
				const res = await importData(parsed);
				if (res.ok && res.data) {
					const r = res.data;
					msg = {
						ok: true,
						text: i18n.t('set.importOk', { c: r.categories, s: r.subscriptions, r: r.reminders })
					};
				} else {
					msg = { ok: false, text: i18n.t('set.importErr') };
				}
			} catch {
				msg = { ok: false, text: i18n.t('set.importInvalid') };
			}
		});
	}
</script>

<div class="actions">
	<button class="primary" onclick={doExport}>
		<Download size={15} />
		{i18n.t('set.export')}
	</button>
	<button class="ghost" onclick={() => fileInput.click()} disabled={importer.on}>
		<Upload size={15} />
		{importer.on ? i18n.t('set.importing') : i18n.t('set.import')}
	</button>
	<input
		bind:this={fileInput}
		type="file"
		accept="application/json,.json"
		onchange={onImportFile}
		hidden
	/>
</div>
<p class="hint">{i18n.t('set.dataHint')}</p>

<FormMessage {msg} />
