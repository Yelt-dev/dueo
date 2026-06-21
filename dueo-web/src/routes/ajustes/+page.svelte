<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Bell, Send, Clock, Database, Users, Lock, Info, Mail } from '@lucide/svelte';
	import {
		me,
		getTelegramStatus,
		setTelegram,
		testTelegram,
		getEmailStatus,
		setEmail,
		testEmail,
		type User
	} from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import Collapsible from '$lib/Collapsible.svelte';
	import PrefsSection from './sections/PrefsSection.svelte';
	import RemindersSection from './sections/RemindersSection.svelte';
	import ChannelSection from './sections/ChannelSection.svelte';
	import type { ChannelConfig } from './sections/channel';
	import DataSection from './sections/DataSection.svelte';
	import UsersSection from './sections/UsersSection.svelte';
	import SecuritySection from './sections/SecuritySection.svelte';
	import AboutSection from './sections/AboutSection.svelte';
	import './settings.css';

	// Accordion: one section open at a time (less scrolling). '' = all closed.
	let openSection = $state('pref');
	const toggle = (id: string) => (openSection = openSection === id ? '' : id);

	// Identity is the only page-level concern: it gates the admin-only Users
	// section and tells it which row is "you". Each section loads its own data.
	let user = $state<User | null>(null);
	const isAdmin = $derived(user?.role === 'admin');
	const meId = $derived(user?.id ?? null);

	onMount(async () => {
		const m = await me();
		if (m.status === 401) return goto('/login');
		if (m.ok && m.data) user = m.data;
	});

	// Telegram and Email share ChannelSection; these adapters normalise each
	// channel's status shape and API calls.
	const telegramConfig: ChannelConfig = {
		icon: Send,
		inputType: 'text',
		placeholder: '-1001234567890',
		fieldKey: 'set.tgChatId',
		hintKey: 'set.tgChatHint',
		enableKey: 'set.tgEnable',
		warnPreKey: 'set.tgNoTokenPre',
		warnCode: 'DUEO_TELEGRAM_BOT_TOKEN',
		testOkKey: 'set.tgTestOk',
		testErrKey: 'set.tgTestErr',
		load: async () => {
			const r = await getTelegramStatus();
			return r.ok && r.data
				? { ready: r.data.bot_ready, enabled: r.data.enabled, value: r.data.chat_id ?? '' }
				: null;
		},
		save: (value, enabled) => setTelegram({ chat_id: value, enabled }),
		test: testTelegram
	};
	const emailConfig: ChannelConfig = {
		icon: Mail,
		inputType: 'email',
		placeholder: 'tu@correo.com',
		fieldKey: 'set.emAddress',
		hintKey: 'set.emHint',
		enableKey: 'set.emEnable',
		warnPreKey: 'set.emNoSmtpPre',
		warnCode: 'DUEO_SMTP_*',
		testOkKey: 'set.emTestOk',
		testErrKey: 'set.emTestErr',
		load: async () => {
			const r = await getEmailStatus();
			return r.ok && r.data
				? { ready: r.data.smtp_ready, enabled: r.data.enabled, value: r.data.email ?? '' }
				: null;
		},
		save: (value, enabled) => setEmail({ email: value, enabled }),
		test: testEmail
	};
</script>

<div class="page settings">
	<header class="bar">
		<h1>{i18n.t('set.title')}</h1>
	</header>

	<Collapsible
		icon={Clock}
		title={i18n.t('set.prefTitle')}
		desc={i18n.t('set.prefDesc')}
		open={openSection === 'pref'}
		ontoggle={() => toggle('pref')}
	>
		<PrefsSection {user} />
	</Collapsible>

	<Collapsible
		icon={Bell}
		title={i18n.t('set.remTitle')}
		desc={i18n.t('set.remDesc')}
		open={openSection === 'rem'}
		ontoggle={() => toggle('rem')}
	>
		<RemindersSection />
	</Collapsible>

	<Collapsible
		icon={Send}
		title={i18n.t('set.tgTitle')}
		desc={i18n.t('set.tgDesc')}
		open={openSection === 'tg'}
		ontoggle={() => toggle('tg')}
	>
		<ChannelSection config={telegramConfig} />
	</Collapsible>

	<Collapsible
		icon={Mail}
		title={i18n.t('set.emTitle')}
		desc={i18n.t('set.emDesc')}
		open={openSection === 'email'}
		ontoggle={() => toggle('email')}
	>
		<ChannelSection config={emailConfig} />
	</Collapsible>

	<Collapsible
		icon={Database}
		title={i18n.t('set.dataTitle')}
		desc={i18n.t('set.dataDesc')}
		open={openSection === 'data'}
		ontoggle={() => toggle('data')}
	>
		<DataSection />
	</Collapsible>

	{#if isAdmin}
		<Collapsible
			icon={Users}
			title={i18n.t('set.usersTitle')}
			desc={i18n.t('set.usersDesc')}
			open={openSection === 'users'}
			ontoggle={() => toggle('users')}
		>
			<UsersSection {meId} />
		</Collapsible>
	{/if}

	<Collapsible
		icon={Lock}
		title={i18n.t('set.secTitle')}
		desc={i18n.t('set.secDesc')}
		open={openSection === 'sec'}
		ontoggle={() => toggle('sec')}
	>
		<SecuritySection />
	</Collapsible>

	<Collapsible
		icon={Info}
		title={i18n.t('set.aboutTitle')}
		desc={i18n.t('set.aboutDesc')}
		open={openSection === 'about'}
		ontoggle={() => toggle('about')}
	>
		<AboutSection />
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
</style>
