<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fade, fly, slide } from 'svelte/transition';
	import {
		LogIn,
		LoaderCircle,
		User,
		Lock,
		UserPlus,
		ShieldCheck,
		Eye,
		EyeOff,
		TriangleAlert
	} from '@lucide/svelte';
	import { login, register, getSetupStatus } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import Logo from '$lib/Logo.svelte';
	import ThemeToggle from '$lib/ThemeToggle.svelte';
	import LangPicker from '$lib/LangPicker.svelte';

	let mode = $state<'login' | 'register'>('login');
	let needsSetup = $state(false); // first account of the instance (= admin)
	let openRegistration = $state(false); // is self-registration allowed after the admin?
	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);
	let showPassword = $state(false);
	let capsOn = $state(false);

	const isRegister = $derived(mode === 'register' || needsSetup);

	// Password strength (guidance only, on register). Scores length and character
	// variety; mapped to Weak/Medium/Strong.
	const strength = $derived(scorePassword(password));
	function scorePassword(p: string) {
		let score = 0;
		if (p.length >= 6) score++;
		if (p.length >= 10) score++;
		if (/[a-z]/.test(p) && /[A-Z]/.test(p)) score++;
		if (/\d/.test(p)) score++;
		if (/[^A-Za-z0-9]/.test(p)) score++;
		if (score <= 1) return { pct: 33, label: i18n.t('login.weak'), color: 'var(--danger)' };
		if (score <= 3) return { pct: 66, label: i18n.t('login.medium'), color: 'var(--warn)' };
		return { pct: 100, label: i18n.t('login.strong'), color: 'var(--ok)' };
	}

	// Caps Lock warning: many login failures are silently caused by it.
	function checkCaps(e: KeyboardEvent) {
		capsOn = e.getModifierState?.('CapsLock') ?? false;
	}

	onMount(async () => {
		try {
			const res = await getSetupStatus();
			if (res.ok && res.data) {
				const s = res.data;
				openRegistration = s.open_registration;
				if (s.needs_setup) {
					needsSetup = true;
					mode = 'register';
				}
			}
		} catch {
			// on failure, stay on the normal login
		}
	});

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		if (isRegister && password.length < 8) {
			error = i18n.t('login.errMin');
			return;
		}
		loading = true;
		try {
			if (isRegister) {
				const res = await register(username.trim(), password);
				if (res.status === 409) {
					error = i18n.t('login.errExists');
					return;
				}
				if (res.status === 403) {
					error = i18n.t('login.errClosed');
					return;
				}
				if (!res.ok) {
					error = i18n.t('login.errCreate');
					return;
				}
				// registered → sign in automatically.
			}
			const res = await login(username.trim(), password);
			if (res.ok) goto('/');
			else if (res.status === 429) error = i18n.t('login.errTooMany');
			else error = isRegister ? i18n.t('login.errNoEntry') : i18n.t('login.errInvalid');
		} catch {
			error = i18n.t('login.errConn');
		} finally {
			loading = false;
		}
	}

	function toggle() {
		mode = mode === 'login' ? 'register' : 'login';
		error = '';
	}
</script>

<main>
	<div class="theme"><LangPicker /><ThemeToggle /></div>
	<form class="card acrylic" onsubmit={submit} in:fly={{ y: 14, duration: 320 }}>
		<Logo size={40} />

		{#if needsSetup}
			<div class="setup">
				<ShieldCheck size={15} />
				<span>{i18n.t('login.setup')}</span>
			</div>
		{:else}
			<p class="sub">
				{isRegister ? i18n.t('login.subtitleRegister') : i18n.t('login.subtitleLogin')}
			</p>
		{/if}

		<div class="field">
			<User size={17} />
			<input
				bind:value={username}
				placeholder={i18n.t('login.user')}
				autocomplete="username"
				aria-label={i18n.t('login.user')}
				required
			/>
		</div>
		<div class="field">
			<Lock size={17} />
			<input
				type={showPassword ? 'text' : 'password'}
				bind:value={password}
				placeholder={isRegister ? i18n.t('login.passwordMin') : i18n.t('login.password')}
				autocomplete={isRegister ? 'new-password' : 'current-password'}
				aria-label={i18n.t('login.password')}
				required
				onkeyup={checkCaps}
				onkeydown={checkCaps}
				onblur={() => (capsOn = false)}
			/>
			<button
				type="button"
				class="eye"
				onclick={() => (showPassword = !showPassword)}
				aria-label={showPassword ? i18n.t('login.hidePass') : i18n.t('login.showPass')}
				tabindex="-1"
			>
				{#if showPassword}<EyeOff size={16} />{:else}<Eye size={16} />{/if}
			</button>
		</div>

		{#if capsOn}
			<p class="caps" transition:fade={{ duration: 150 }}>
				<TriangleAlert size={13} />
				{i18n.t('login.capsOn')}
			</p>
		{/if}

		{#if isRegister && password.length > 0}
			<div class="strength" transition:slide={{ duration: 160 }}>
				<div class="track">
					<div class="fill" style="width:{strength.pct}%; background:{strength.color}"></div>
				</div>
				<span style="color:{strength.color}">{strength.label}</span>
			</div>
		{/if}

		{#if error}<p class="err" transition:fade={{ duration: 150 }}>{error}</p>{/if}

		<button type="submit" class="submit" disabled={loading}>
			{#if loading}
				<LoaderCircle size={16} class="spin" />
			{:else if isRegister}
				<UserPlus size={16} />
			{:else}
				<LogIn size={16} />
			{/if}
			{isRegister ? i18n.t('login.createAccount') : i18n.t('login.signIn')}
		</button>

		{#if !needsSetup && openRegistration}
			<button type="button" class="toggle" onclick={toggle}>
				{mode === 'login' ? i18n.t('login.toRegister') : i18n.t('login.toLogin')}
			</button>
		{/if}
	</form>
</main>

<style>
	main {
		position: relative;
		min-height: 100dvh;
		display: grid;
		place-items: center;
		padding: 1.5rem;
	}
	.theme {
		position: absolute;
		top: 1.25rem;
		right: 1.25rem;
		display: flex;
		gap: 0.5rem;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		width: 100%;
		max-width: 360px;
		padding: 2rem;
		border-radius: var(--radius-xl, 20px);
	}
	.sub {
		margin: -0.25rem 0 0.75rem;
		color: var(--text-muted);
		font-size: 0.85rem;
	}
	.setup {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin: -0.25rem 0 0.75rem;
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		font-size: 0.8rem;
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 30%, transparent);
	}

	/* input with icon inside */
	.field {
		display: flex;
		align-items: center;
		gap: 0.55rem;
		padding: 0 0.75rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-muted);
		transition: border-color 0.15s;
	}
	.field:focus-within {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
	}
	.field input {
		flex: 1;
		min-width: 0;
		border: none;
		background: transparent;
		padding: 0.62rem 0;
		color: var(--text);
		font-size: 0.95rem;
	}
	.field input:focus {
		outline: none;
	}
	.field input::placeholder {
		color: var(--text-muted);
	}

	/* eye toggle button inside the field */
	.eye {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		margin-left: 0.25rem;
		border: none;
		border-radius: 7px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: color 0.15s;
	}
	.eye:hover {
		color: var(--text);
	}

	.caps {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		margin: 0;
		color: var(--warn);
		font-size: 0.78rem;
	}

	.strength {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-size: 0.76rem;
		font-weight: 600;
	}
	.strength .track {
		flex: 1;
		height: 5px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text) 12%, transparent);
		overflow: hidden;
	}
	.strength .fill {
		height: 100%;
		border-radius: 999px;
		transition:
			width 0.2s ease,
			background 0.2s ease;
	}

	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}

	.submit {
		margin-top: 0.5rem;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		height: 42px;
		border: none;
		border-radius: 11px;
		font-weight: 650;
		font-size: 0.95rem;
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
		cursor: pointer;
		transition: transform 0.15s;
	}
	.submit:hover {
		transform: translateY(-1px);
	}
	.submit:disabled {
		opacity: 0.6;
		cursor: default;
		transform: none;
	}
	.toggle {
		margin-top: 0.25rem;
		border: none;
		background: transparent;
		color: var(--text-2);
		font-size: 0.82rem;
		cursor: pointer;
	}
	.toggle:hover {
		color: var(--brand);
		text-decoration: underline;
	}
	:global(.spin) {
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
