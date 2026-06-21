<script lang="ts">
	import { goto } from '$app/navigation';
	import { Check, X } from '@lucide/svelte';
	import { changePassword, logoutAll } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import { busy } from './busy.svelte';
	import FormMessage from './FormMessage.svelte';

	let curPass = $state('');
	let newPass = $state('');
	let newPass2 = $state('');
	let msg = $state<{ ok: boolean; text: string } | null>(null);
	const saver = busy();
	const loggingOut = busy();

	function savePassword() {
		msg = null;
		if (newPass.length < 8) return (msg = { ok: false, text: i18n.t('set.pwErrMin') });
		if (newPass !== newPass2) return (msg = { ok: false, text: i18n.t('set.pwErrMatch') });
		saver.run(async () => {
			const res = await changePassword({ current_password: curPass, new_password: newPass });
			if (res.ok) {
				msg = { ok: true, text: i18n.t('set.pwOk') };
				curPass = newPass = newPass2 = '';
			} else {
				msg = { ok: false, text: i18n.t('set.pwErr') };
			}
		});
	}

	function closeAllSessions() {
		loggingOut.run(async () => {
			await logoutAll();
			goto('/login');
		});
	}
</script>

<form class="pwform" onsubmit={(e) => (e.preventDefault(), savePassword())}>
	<label class="field">
		{i18n.t('set.currentPass')}
		<input bind:value={curPass} type="password" autocomplete="current-password" />
	</label>
	<div class="grid2">
		<label class="field">
			{i18n.t('set.newPass')}
			<input bind:value={newPass} type="password" autocomplete="new-password" />
		</label>
		<label class="field">
			{i18n.t('set.repeatPass')}
			<input bind:value={newPass2} type="password" autocomplete="new-password" />
		</label>
	</div>
	<div class="actions">
		<button type="submit" class="primary" disabled={saver.on || !curPass || !newPass}>
			<Check size={15} />
			{saver.on ? i18n.t('common.saving') : i18n.t('set.changePass')}
		</button>
	</div>
</form>
<FormMessage {msg} />

<div class="sep"></div>
<div class="actions">
	<button class="ghost" onclick={closeAllSessions} disabled={loggingOut.on}>
		<X size={15} />
		{loggingOut.on ? i18n.t('set.closing') : i18n.t('set.closeAll')}
	</button>
</div>
<p class="hint">{i18n.t('set.closeAllHint')}</p>

<style>
	.pwform {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
	}
	.sep {
		height: 1px;
		background: var(--border);
		margin: 0.2rem 0;
	}
</style>
