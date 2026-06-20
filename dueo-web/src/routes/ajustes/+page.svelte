<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fade, slide } from 'svelte/transition';
	import {
		Bell,
		Send,
		Plus,
		X,
		Check,
		TriangleAlert,
		Clock,
		Database,
		Download,
		Upload,
		Users,
		Trash2,
		Lock,
		Info,
		ExternalLink,
		Mail
	} from '@lucide/svelte';
	import {
		getReminders,
		createReminder,
		deleteReminder,
		getTelegramStatus,
		setTelegram,
		testTelegram,
		getEmailStatus,
		setEmail,
		testEmail,
		me,
		updateSettings,
		exportData,
		importData,
		getUsers,
		createUser,
		deleteUser,
		changePassword,
		logoutAll,
		getVersion,
		getHealth,
		type Reminder,
		type TelegramStatus,
		type EmailStatus,
		type AdminUser,
		type VersionInfo
	} from '$lib/api';
	import { i18n, reminderLabel } from '$lib/i18n.svelte';
	import Collapsible from '$lib/Collapsible.svelte';

	// Acordeón: una sección abierta a la vez (menos scroll). '' = todas cerradas.
	let openSection = $state('pref');
	const toggle = (id: string) => (openSection = openSection === id ? '' : id);

	// --- Usuario actual (rol) ---
	let role = $state('member');
	let meId = $state<number | null>(null);
	const isAdmin = $derived(role === 'admin');

	// --- Datos (export / import) ---
	let importing = $state(false);
	let dataMsg = $state<{ ok: boolean; text: string } | null>(null);
	let fileInput: HTMLInputElement;

	// --- Usuarios (admin) ---
	let users = $state<AdminUser[]>([]);
	let usersLoading = $state(true);
	let newUsername = $state('');
	let newPassword = $state('');
	let newRole = $state('member');
	let creatingUser = $state(false);
	let usersErr = $state('');
	let confirmDelUser = $state<number | null>(null);

	// --- Seguridad (contraseña + sesiones) ---
	let curPass = $state('');
	let newPass = $state('');
	let newPass2 = $state('');
	let pwSaving = $state(false);
	let pwMsg = $state<{ ok: boolean; text: string } | null>(null);
	let loggingOut = $state(false);

	// --- Acerca de ---
	let versionInfo = $state<VersionInfo | null>(null);
	let backendOk = $state<boolean | null>(null);

	// --- Recordatorios (solo las reglas GLOBALES del usuario) ---
	let reminders = $state<Reminder[]>([]);
	const globals = $derived(
		reminders
			.filter((r) => r.subscription_id === null)
			.sort((a, b) => a.days_before - b.days_before)
	);
	let newDays = $state('');
	let remErr = $state('');

	// --- Preferencias (zona horaria + hora de aviso) ---
	// Zonas comunes (LATAM + algunas) + UTC. El backend valida contra IANA.
	const TIMEZONES = [
		'UTC',
		'America/Lima',
		'America/Bogota',
		'America/Mexico_City',
		'America/Argentina/Buenos_Aires',
		'America/Santiago',
		'America/Caracas',
		'America/Sao_Paulo',
		'America/New_York',
		'America/Los_Angeles',
		'Europe/Madrid'
	];
	let timezone = $state('UTC');
	let sendHour = $state(9);
	let defaultCurrency = $state('USD');
	const CURRENCIES = [
		'USD',
		'EUR',
		'GBP',
		'MXN',
		'ARS',
		'COP',
		'CLP',
		'PEN',
		'BRL',
		'UYU',
		'BOB',
		'PYG',
		'VES',
		'CRC',
		'GTQ',
		'DOP',
		'CAD',
		'JPY',
		'CNY',
		'CHF',
		'AUD'
	];
	let prefSaving = $state(false);
	let prefMsg = $state('');

	// --- Telegram ---
	let tg = $state<TelegramStatus | null>(null);
	let chatId = $state('');
	let enabled = $state(false);
	let saving = $state(false);
	let testing = $state(false);
	let tgMsg = $state<{ ok: boolean; text: string } | null>(null);

	// --- Email ---
	let em = $state<EmailStatus | null>(null);
	let emailAddr = $state('');
	let emEnabled = $state(false);
	let emSaving = $state(false);
	let emTesting = $state(false);
	let emMsg = $state<{ ok: boolean; text: string } | null>(null);

	onMount(async () => {
		const r = await getReminders();
		if (r.status === 401) return goto('/login');
		if (r.ok) reminders = await r.json();

		const m = await me();
		if (m.ok) {
			const u = await m.json();
			timezone = u.timezone ?? 'UTC';
			sendHour = u.send_hour ?? 9;
			defaultCurrency = u.default_currency ?? 'USD';
			role = u.role ?? 'member';
			meId = u.id ?? null;
		}

		const t = await getTelegramStatus();
		if (t.ok) {
			tg = await t.json();
			chatId = tg!.chat_id ?? '';
			enabled = tg!.enabled;
		}

		const e = await getEmailStatus();
		if (e.ok) {
			em = await e.json();
			emailAddr = em!.email ?? '';
			emEnabled = em!.enabled;
		}

		// Usuarios (solo si admin) — se cargan aparte para no bloquear el resto.
		if (isAdmin) {
			await loadUsers();
		} else {
			usersLoading = false;
		}

		// Acerca de: versión + salud del backend.
		const v = await getVersion();
		if (v.ok) versionInfo = await v.json();
		try {
			const h = await getHealth();
			backendOk = h.ok;
		} catch {
			backendOk = false;
		}
	});

	async function loadUsers() {
		usersLoading = true;
		const res = await getUsers();
		if (res.ok) users = await res.json();
		usersLoading = false;
	}

	// --- Datos ---
	async function doExport() {
		dataMsg = null;
		const res = await exportData();
		if (!res.ok) return (dataMsg = { ok: false, text: i18n.t('set.exportErr') });
		const data = await res.json();
		const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		const date = new Date().toISOString().slice(0, 10);
		a.href = url;
		a.download = `dueo-backup-${date}.json`;
		a.click();
		URL.revokeObjectURL(url);
		dataMsg = { ok: true, text: i18n.t('set.exportOk') };
	}

	async function onImportFile(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = ''; // permite re-elegir el mismo archivo
		if (!file) return;
		importing = true;
		dataMsg = null;
		try {
			const text = await file.text();
			const parsed = JSON.parse(text);
			const res = await importData(parsed);
			if (res.ok) {
				const r = await res.json();
				dataMsg = {
					ok: true,
					text: i18n.t('set.importOk', { c: r.categories, s: r.subscriptions, r: r.reminders })
				};
			} else {
				dataMsg = { ok: false, text: (await res.text()) || i18n.t('set.importErr') };
			}
		} catch {
			dataMsg = { ok: false, text: i18n.t('set.importInvalid') };
		} finally {
			importing = false;
		}
	}

	// --- Usuarios ---
	async function addUser() {
		usersErr = '';
		if (!newUsername.trim()) return (usersErr = i18n.t('set.usersErrName'));
		if (newPassword.length < 8) return (usersErr = i18n.t('set.usersErrPass'));
		creatingUser = true;
		try {
			const res = await createUser({
				username: newUsername.trim(),
				password: newPassword,
				role: newRole
			});
			if (res.ok) {
				users = [...users, await res.json()];
				newUsername = '';
				newPassword = '';
				newRole = 'member';
			} else {
				usersErr = (await res.text()) || 'No se pudo crear';
			}
		} finally {
			creatingUser = false;
		}
	}

	async function removeUser(u: AdminUser) {
		const res = await deleteUser(u.id);
		if (res.ok || res.status === 404) {
			users = users.filter((x) => x.id !== u.id);
		} else {
			usersErr = (await res.text()) || 'No se pudo borrar';
		}
		confirmDelUser = null;
	}

	// --- Seguridad ---
	async function savePassword() {
		pwMsg = null;
		if (newPass.length < 8) return (pwMsg = { ok: false, text: i18n.t('set.pwErrMin') });
		if (newPass !== newPass2) return (pwMsg = { ok: false, text: i18n.t('set.pwErrMatch') });
		pwSaving = true;
		try {
			const res = await changePassword({ current_password: curPass, new_password: newPass });
			if (res.ok) {
				pwMsg = { ok: true, text: i18n.t('set.pwOk') };
				curPass = newPass = newPass2 = '';
			} else {
				pwMsg = { ok: false, text: (await res.text()) || i18n.t('set.pwErr') };
			}
		} finally {
			pwSaving = false;
		}
	}

	async function closeAllSessions() {
		loggingOut = true;
		await logoutAll();
		goto('/login');
	}

	async function savePrefs() {
		prefSaving = true;
		prefMsg = '';
		try {
			const res = await updateSettings({
				timezone,
				send_hour: sendHour,
				default_currency: defaultCurrency
			});
			prefMsg = res.ok ? i18n.t('set.savedOk') : i18n.t('set.saveErr');
		} finally {
			prefSaving = false;
		}
	}

	async function addRule() {
		remErr = '';
		const n = parseInt(newDays, 10);
		if (isNaN(n) || n < 0) return (remErr = i18n.t('set.remErrInvalid'));
		if (globals.some((g) => g.days_before === n)) return (remErr = i18n.t('set.remErrExists'));
		const res = await createReminder({ subscription_id: null, days_before: n });
		if (res.ok) {
			reminders = [...reminders, await res.json()];
			newDays = '';
		} else {
			remErr = i18n.t('set.remErrAdd');
		}
	}

	async function removeRule(r: Reminder) {
		const res = await deleteReminder(r.id);
		if (res.ok || res.status === 404) reminders = reminders.filter((x) => x.id !== r.id);
	}

	async function saveTelegram() {
		saving = true;
		tgMsg = null;
		try {
			const res = await setTelegram({ chat_id: chatId.trim(), enabled });
			if (res.ok && tg) tg = { ...tg, chat_id: chatId.trim(), enabled };
			else if (!res.ok) tgMsg = { ok: false, text: i18n.t('set.saveErr') };
		} finally {
			saving = false;
		}
	}

	async function sendTest() {
		testing = true;
		tgMsg = null;
		try {
			const res = await testTelegram();
			if (res.ok) tgMsg = { ok: true, text: i18n.t('set.tgTestOk') };
			else tgMsg = { ok: false, text: (await res.text()) || i18n.t('set.tgTestErr') };
		} catch {
			tgMsg = { ok: false, text: i18n.t('common.connError') };
		} finally {
			testing = false;
		}
	}

	async function saveEmail() {
		emSaving = true;
		emMsg = null;
		try {
			const res = await setEmail({ email: emailAddr.trim(), enabled: emEnabled });
			if (res.ok && em) em = { ...em, email: emailAddr.trim(), enabled: emEnabled };
			else if (!res.ok) emMsg = { ok: false, text: (await res.text()) || i18n.t('set.saveErr') };
		} finally {
			emSaving = false;
		}
	}

	async function sendEmailTest() {
		emTesting = true;
		emMsg = null;
		try {
			const res = await testEmail();
			if (res.ok) emMsg = { ok: true, text: i18n.t('set.emTestOk') };
			else emMsg = { ok: false, text: (await res.text()) || i18n.t('set.emTestErr') };
		} catch {
			emMsg = { ok: false, text: i18n.t('common.connError') };
		} finally {
			emTesting = false;
		}
	}
</script>

<div class="page">
	<header class="bar">
		<h1>{i18n.t('set.title')}</h1>
	</header>

	<!-- Preferencias -->
	<Collapsible
		icon={Clock}
		title={i18n.t('set.prefTitle')}
		desc={i18n.t('set.prefDesc')}
		open={openSection === 'pref'}
		ontoggle={() => toggle('pref')}
	>
		<div class="grid2">
			<label class="field">
				{i18n.t('set.timezone')}
				<select bind:value={timezone}>
					{#if !TIMEZONES.includes(timezone)}<option value={timezone}>{timezone}</option>{/if}
					{#each TIMEZONES as tz (tz)}<option value={tz}>{tz}</option>{/each}
				</select>
			</label>
			<label class="field">
				{i18n.t('set.sendHour')}
				<select bind:value={sendHour}>
					{#each Array(24) as _, h (h)}
						<option value={h}>{String(h).padStart(2, '0')}:00</option>
					{/each}
				</select>
			</label>
			<label class="field">
				{i18n.t('set.mainCurrency')}
				<select bind:value={defaultCurrency}>
					{#if !CURRENCIES.includes(defaultCurrency)}<option value={defaultCurrency}
							>{defaultCurrency}</option
						>{/if}
					{#each CURRENCIES as c (c)}<option value={c}>{c}</option>{/each}
				</select>
			</label>
		</div>

		<div class="actions">
			<button class="primary" onclick={savePrefs} disabled={prefSaving}>
				<Check size={15} />
				{prefSaving ? i18n.t('common.saving') : i18n.t('common.save')}
			</button>
			{#if prefMsg}<span class="ok" style="align-self:center">{prefMsg}</span>{/if}
		</div>
	</Collapsible>

	<!-- Recordatorios -->
	<Collapsible
		icon={Bell}
		title={i18n.t('set.remTitle')}
		desc={i18n.t('set.remDesc')}
		open={openSection === 'rem'}
		ontoggle={() => toggle('rem')}
	>
		<div class="chips">
			{#each globals as g (g.id)}
				<span class="chip">
					{reminderLabel(g.days_before)}
					<button onclick={() => removeRule(g)} aria-label={i18n.t('common.delete')}
						><X size={13} /></button
					>
				</span>
			{/each}
			{#if globals.length === 0}
				<span class="muted">{i18n.t('set.remEmpty')}</span>
			{/if}
		</div>

		<form class="addrule" onsubmit={(e) => (e.preventDefault(), addRule())}>
			<input bind:value={newDays} inputmode="numeric" placeholder={i18n.t('set.remPlaceholder')} />
			<button type="submit" class="ghost"><Plus size={15} /> {i18n.t('common.add')}</button>
		</form>
		{#if remErr}<p class="err">{remErr}</p>{/if}
	</Collapsible>

	<!-- Telegram -->
	<Collapsible
		icon={Send}
		title={i18n.t('set.tgTitle')}
		desc={i18n.t('set.tgDesc')}
		open={openSection === 'tg'}
		ontoggle={() => toggle('tg')}
	>
		{#if tg && !tg.bot_ready}
			<div class="warn">
				<TriangleAlert size={15} />
				{i18n.t('set.tgNoTokenPre')}<code>DUEO_TELEGRAM_BOT_TOKEN</code>).
			</div>
		{/if}

		<label class="field">
			{i18n.t('set.tgChatId')}
			<input bind:value={chatId} placeholder="-1001234567890" />
			<span class="hint">{i18n.t('set.tgChatHint')}</span>
		</label>

		<label class="check">
			<input type="checkbox" bind:checked={enabled} />
			{i18n.t('set.tgEnable')}
		</label>

		<div class="actions">
			<button class="primary" onclick={saveTelegram} disabled={saving}>
				<Check size={15} />
				{saving ? i18n.t('common.saving') : i18n.t('common.save')}
			</button>
			<button
				class="ghost"
				onclick={sendTest}
				disabled={testing || !tg?.bot_ready || !chatId.trim()}
			>
				<Send size={15} />
				{testing ? i18n.t('set.tgTesting') : i18n.t('set.tgTest')}
			</button>
		</div>

		{#if tgMsg}
			<p transition:slide={{ duration: 180 }} class:err={!tgMsg.ok} class:ok={tgMsg.ok}>
				{tgMsg.text}
			</p>
		{/if}
	</Collapsible>

	<!-- Email -->
	<Collapsible
		icon={Mail}
		title={i18n.t('set.emTitle')}
		desc={i18n.t('set.emDesc')}
		open={openSection === 'email'}
		ontoggle={() => toggle('email')}
	>
		{#if em && !em.smtp_ready}
			<div class="warn">
				<TriangleAlert size={15} />
				{i18n.t('set.emNoSmtpPre')}<code>DUEO_SMTP_*</code>).
			</div>
		{/if}

		<label class="field">
			{i18n.t('set.emAddress')}
			<input bind:value={emailAddr} type="email" placeholder="tu@correo.com" />
			<span class="hint">{i18n.t('set.emHint')}</span>
		</label>

		<label class="check">
			<input type="checkbox" bind:checked={emEnabled} />
			{i18n.t('set.emEnable')}
		</label>

		<div class="actions">
			<button class="primary" onclick={saveEmail} disabled={emSaving}>
				<Check size={15} />
				{emSaving ? i18n.t('common.saving') : i18n.t('common.save')}
			</button>
			<button
				class="ghost"
				onclick={sendEmailTest}
				disabled={emTesting || !em?.smtp_ready || !emailAddr.trim()}
			>
				<Mail size={15} />
				{emTesting ? i18n.t('set.tgTesting') : i18n.t('set.tgTest')}
			</button>
		</div>

		{#if emMsg}
			<p transition:slide={{ duration: 180 }} class:err={!emMsg.ok} class:ok={emMsg.ok}>
				{emMsg.text}
			</p>
		{/if}
	</Collapsible>

	<!-- Datos (export / import) -->
	<Collapsible
		icon={Database}
		title={i18n.t('set.dataTitle')}
		desc={i18n.t('set.dataDesc')}
		open={openSection === 'data'}
		ontoggle={() => toggle('data')}
	>
		<div class="actions">
			<button class="primary" onclick={doExport}>
				<Download size={15} />
				{i18n.t('set.export')}
			</button>
			<button class="ghost" onclick={() => fileInput.click()} disabled={importing}>
				<Upload size={15} />
				{importing ? i18n.t('set.importing') : i18n.t('set.import')}
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

		{#if dataMsg}
			<p transition:slide={{ duration: 180 }} class:err={!dataMsg.ok} class:ok={dataMsg.ok}>
				{dataMsg.text}
			</p>
		{/if}
	</Collapsible>

	<!-- Usuarios (solo admin) -->
	{#if isAdmin}
		<Collapsible
			icon={Users}
			title={i18n.t('set.usersTitle')}
			desc={i18n.t('set.usersDesc')}
			open={openSection === 'users'}
			ontoggle={() => toggle('users')}
		>
			{#if usersLoading}
				<div class="userlist">
					{#each Array(2) as _, i (i)}
						<div class="urow">
							<div class="skeleton" style="width:40%;height:1rem"></div>
							<div class="skeleton" style="width:64px;height:1rem"></div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="userlist">
					{#each users as u (u.id)}
						<div class="urow" transition:slide={{ duration: 160 }}>
							<div class="uinfo">
								<span class="uname">{u.username}</span>
								<span class="urole" class:admin={u.role === 'admin'}
									>{u.role === 'admin' ? i18n.t('set.roleAdmin') : i18n.t('set.roleMember')}</span
								>
								{#if u.id === meId}<span class="ume">{i18n.t('set.you')}</span>{/if}
							</div>
							{#if u.id !== meId}
								{#if confirmDelUser === u.id}
									<div class="confirm" transition:fade={{ duration: 120 }}>
										<button class="danger" onclick={() => removeUser(u)}
											>{i18n.t('common.delete')}</button
										>
										<button class="ghost sm" onclick={() => (confirmDelUser = null)}
											>{i18n.t('common.cancel')}</button
										>
									</div>
								{:else}
									<button
										class="icon-btn"
										aria-label={i18n.t('set.deleteUser')}
										onclick={() => (confirmDelUser = u.id)}
									>
										<Trash2 size={15} />
									</button>
								{/if}
							{/if}
						</div>
					{/each}
				</div>
			{/if}

			<form class="newuser" onsubmit={(e) => (e.preventDefault(), addUser())}>
				<input
					bind:value={newUsername}
					placeholder={i18n.t('set.userPlaceholder')}
					autocomplete="off"
				/>
				<input
					bind:value={newPassword}
					type="password"
					placeholder={i18n.t('set.passPlaceholder')}
					autocomplete="new-password"
				/>
				<select bind:value={newRole}>
					<option value="member">{i18n.t('set.roleMember')}</option>
					<option value="admin">{i18n.t('set.roleAdmin')}</option>
				</select>
				<button type="submit" class="ghost" disabled={creatingUser}>
					<Plus size={15} />
					{creatingUser ? i18n.t('set.creating') : i18n.t('common.create')}
				</button>
			</form>
			{#if usersErr}<p transition:slide={{ duration: 180 }} class="err">{usersErr}</p>{/if}
		</Collapsible>
	{/if}

	<!-- Seguridad -->
	<Collapsible
		icon={Lock}
		title={i18n.t('set.secTitle')}
		desc={i18n.t('set.secDesc')}
		open={openSection === 'sec'}
		ontoggle={() => toggle('sec')}
	>
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
				<button type="submit" class="primary" disabled={pwSaving || !curPass || !newPass}>
					<Check size={15} />
					{pwSaving ? i18n.t('common.saving') : i18n.t('set.changePass')}
				</button>
			</div>
		</form>
		{#if pwMsg}
			<p transition:slide={{ duration: 180 }} class:err={!pwMsg.ok} class:ok={pwMsg.ok}>
				{pwMsg.text}
			</p>
		{/if}

		<div class="sep"></div>
		<div class="actions">
			<button class="ghost" onclick={closeAllSessions} disabled={loggingOut}>
				<X size={15} />
				{loggingOut ? i18n.t('set.closing') : i18n.t('set.closeAll')}
			</button>
		</div>
		<p class="hint">{i18n.t('set.closeAllHint')}</p>
	</Collapsible>

	<!-- Acerca de -->
	<Collapsible
		icon={Info}
		title={i18n.t('set.aboutTitle')}
		desc={i18n.t('set.aboutDesc')}
		open={openSection === 'about'}
		ontoggle={() => toggle('about')}
	>
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
	</Collapsible>
</div>

<style>
	.bar {
		margin-bottom: var(--gap-section);
	}
	.bar h1 {
		margin: 0;
		font-size: 1.3rem;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		align-items: center;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 6px 5px 11px;
		border-radius: 999px;
		font-size: 0.82rem;
		font-weight: 550;
		color: var(--text);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 30%, transparent);
	}
	.chip button {
		display: grid;
		place-items: center;
		width: 18px;
		height: 18px;
		border: none;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text) 10%, transparent);
		color: var(--text-2);
		cursor: pointer;
	}
	.chip button:hover {
		color: var(--text);
		background: color-mix(in srgb, var(--danger) 22%, transparent);
	}
	.muted {
		font-size: 0.82rem;
		color: var(--text-muted);
	}
	.addrule {
		display: flex;
		gap: 0.5rem;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		font-size: 0.8rem;
		color: var(--text-2);
	}
	.field .hint {
		font-size: 0.74rem;
		color: var(--text-muted);
	}
	input:not([type]),
	input[type='email'],
	.addrule input,
	select {
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.9rem;
	}
	.addrule input {
		flex: 1;
	}
	input:focus-visible,
	select:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
	}
	.grid2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.7rem;
	}
	@media (max-width: 520px) {
		.grid2 {
			grid-template-columns: 1fr;
		}
	}
	/* Móvil: los botones de acción aprovechan el ancho. Uno solo → full width;
	   dos → dos columnas al 50% que llenan la fila. */
	@media (max-width: 640px) {
		.actions {
			flex-wrap: wrap;
		}
		.actions > button {
			flex: 1 1 0;
			justify-content: center;
		}
	}
	.check {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.88rem;
		color: var(--text);
		cursor: pointer;
	}
	.check input {
		width: 16px;
		height: 16px;
		accent-color: var(--brand);
	}
	.warn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.6rem 0.8rem;
		border-radius: 10px;
		font-size: 0.8rem;
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--warn) 30%, transparent);
	}
	.warn code {
		font-size: 0.76rem;
	}
	.actions {
		display: flex;
		gap: 0.6rem;
	}
	.primary,
	.ghost {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 38px;
		padding: 0 1rem;
		border-radius: 11px;
		font-weight: 600;
		font-size: 0.86rem;
		cursor: pointer;
	}
	.primary {
		border: none;
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.ghost {
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
	}
	.ghost:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	.primary:disabled,
	.ghost:disabled {
		opacity: 0.55;
		cursor: default;
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
	.ok {
		margin: 0;
		color: var(--ok);
		font-size: 0.82rem;
	}
	/* hint suelto (no solo dentro de .field) */
	p.hint {
		margin: 0;
		font-size: 0.76rem;
		color: var(--text-muted);
	}

	/* --- Usuarios --- */
	.userlist {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.urow {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.6rem;
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		background: var(--surface-2);
		border: 1px solid var(--border);
	}
	.uinfo {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
	}
	.uname {
		font-size: 0.9rem;
		font-weight: 550;
		color: var(--text);
	}
	.urole {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 999px;
		color: var(--text-2);
		background: color-mix(in srgb, var(--text) 8%, transparent);
		text-transform: capitalize;
	}
	.urole.admin {
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
	}
	.ume {
		font-size: 0.7rem;
		color: var(--text-muted);
	}
	.confirm {
		display: flex;
		gap: 0.4rem;
	}
	.icon-btn {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}
	.icon-btn:hover {
		color: var(--danger);
		border-color: color-mix(in srgb, var(--danger) 40%, transparent);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
	}
	.danger {
		display: inline-flex;
		align-items: center;
		height: 32px;
		padding: 0 0.8rem;
		border: none;
		border-radius: 9px;
		font-size: 0.82rem;
		font-weight: 600;
		color: white;
		background: var(--danger);
		cursor: pointer;
	}
	.ghost.sm,
	.danger {
		height: 32px;
	}
	.ghost.sm {
		padding: 0 0.7rem;
		font-size: 0.82rem;
	}
	.newuser {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}
	.newuser input {
		flex: 1;
		min-width: 130px;
	}
	.newuser input,
	.newuser select {
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.9rem;
	}
	/* "Crear" salta a su propia fila, a todo el ancho, como CTA claro. */
	.newuser button {
		flex: 1 0 100%;
		justify-content: center;
	}

	/* --- Seguridad --- */
	.pwform {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
	}
	.pwform input {
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.9rem;
	}
	.sep {
		height: 1px;
		background: var(--border);
		margin: 0.2rem 0;
	}

	/* --- Acerca de --- */
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
