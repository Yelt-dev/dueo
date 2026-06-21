<script lang="ts">
	import { onMount } from 'svelte';
	import { Bell, CheckCheck, Inbox, X } from '@lucide/svelte';
	import { fly } from 'svelte/transition';
	import {
		getNotifications,
		markNotificationRead,
		markAllNotificationsRead,
		type Notification
	} from './api';
	import { i18n } from './i18n.svelte';
	import { portal } from './actions';
	import Popover from './Popover.svelte';

	let items = $state<Notification[]>([]);
	let open = $state(false);
	let toasts = $state<Notification[]>([]); // transient toasts (live arrivals)

	// The panel shows only the in-app channel ('telegram' rows are the same
	// notification delivered over another channel: no UI duplication).
	const inapp = $derived(items.filter((n) => n.channel === 'inapp'));
	const unread = $derived(inapp.filter((n) => !n.read_at).length);

	async function load() {
		const res = await getNotifications();
		if (res.ok && res.data) items = res.data;
	}

	onMount(() => {
		load();

		// Live notifications over SSE (same origin → the session cookie rides along
		// and the backend filters by user). Reconnect with backoff if the stream
		// actually closes (e.g. the cookie expired), refetching what we missed.
		let es: EventSource | null = null;
		let retry = 0;

		function connect() {
			es = new EventSource('/api/notifications/stream');
			es.onopen = () => (retry = 0);
			es.onmessage = (e) => {
				try {
					const n = JSON.parse(e.data);
					if (n.channel !== 'inapp') return; // only the in-app channel feeds the bell
					if (items.some((x) => x.id === n.id)) return; // already have it
					items = [n, ...items];
					showToast(n);
				} catch {
					// ignore non-JSON payloads (keep-alives)
				}
			};
			es.onerror = () => {
				// The browser auto-retries an open stream; only step in once it has
				// truly closed, then back off (cap 30s) and resync the list.
				if (es?.readyState === EventSource.CLOSED) {
					es = null;
					const delay = Math.min(30_000, 1000 * 2 ** retry++);
					setTimeout(() => {
						load();
						connect();
					}, delay);
				}
			};
		}

		connect();
		return () => es?.close();
	});

	function showToast(n: Notification) {
		toasts = [n, ...toasts].slice(0, 4); // at most 4 at once
		setTimeout(() => dismissToast(n.id), 6000); // auto-dismiss
	}
	function dismissToast(id: number) {
		toasts = toasts.filter((t) => t.id !== id);
	}

	async function markAll() {
		await markAllNotificationsRead();
		const now = new Date().toISOString();
		items = items.map((n) => ({ ...n, read_at: n.read_at ?? now }));
	}

	async function onItem(n: Notification) {
		if (n.read_at) return;
		await markNotificationRead(n.id);
		const now = new Date().toISOString();
		items = items.map((x) => (x.id === n.id ? { ...x, read_at: now } : x));
	}

	// Simple "3h ago", no library.
	function ago(iso: string): string {
		// Backend stores naive UTC ('YYYY-MM-DD HH:MM:SS'); treat it as UTC.
		const t = new Date(iso.replace(' ', 'T') + 'Z').getTime();
		const s = Math.max(0, Math.floor((Date.now() - t) / 1000));
		if (s < 60) return i18n.t('ago.now');
		const m = Math.floor(s / 60);
		if (m < 60) return i18n.t('ago.min', { n: m });
		const h = Math.floor(m / 60);
		if (h < 24) return i18n.t('ago.hour', { n: h });
		const d = Math.floor(h / 24);
		return i18n.t('ago.day', { n: d });
	}
</script>

<Popover
	bind:open
	onopen={load}
	style="--pop-width: 340px; --pop-min-width: 0; --pop-pad: 0; --pop-gap-items: 0; --pop-radius: var(--radius-lg, 16px); --pop-overflow: hidden; --pop-shadow: 0 18px 44px -16px rgba(0, 0, 0, 0.6)"
>
	{#snippet trigger({ open, toggle })}
		<button
			class="icon"
			onclick={toggle}
			aria-label={i18n.t('notif.title')}
			aria-haspopup="menu"
			aria-expanded={open}
		>
			<Bell size={18} />
			{#if unread > 0}<span class="badge">{unread > 9 ? '9+' : unread}</span>{/if}
		</button>
	{/snippet}

	<header>
		<span class="title">{i18n.t('notif.title')}</span>
		{#if unread > 0}
			<button class="markall" onclick={markAll}
				><CheckCheck size={14} /> {i18n.t('notif.markAll')}</button
			>
		{/if}
	</header>

	{#if inapp.length === 0}
		<div class="empty">
			<Inbox size={28} />
			<p>{i18n.t('notif.empty')}</p>
		</div>
	{:else}
		<ul>
			{#each inapp as n (n.id)}
				<li>
					<button class="item" class:unread={!n.read_at} onclick={() => onItem(n)}>
						{#if !n.read_at}<span class="dot"></span>{/if}
						<span class="msg">{n.message}</span>
						<span class="time">{ago(n.created_at)}</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</Popover>

<!-- Toasts (live arrivals). use:portal → mounted on <body>, fixed to the viewport. -->
{#if toasts.length}
	<div class="toasts" use:portal aria-live="polite">
		{#each toasts as t (t.id)}
			<button
				type="button"
				class="toast"
				onclick={() => dismissToast(t.id)}
				transition:fly={{ x: 24, duration: 240 }}
			>
				<Bell size={16} />
				<span class="tmsg">{t.message}</span>
				<span class="tclose"><X size={14} /></span>
			</button>
		{/each}
	</div>
{/if}

<style>
	.icon {
		position: relative;
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s,
			transform 0.15s;
	}
	.icon:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}
	.badge {
		position: absolute;
		top: -4px;
		right: -4px;
		min-width: 17px;
		height: 17px;
		padding: 0 4px;
		display: grid;
		place-items: center;
		font-size: 0.62rem;
		font-weight: 700;
		color: white;
		background: var(--danger);
		border-radius: 999px;
		border: 2px solid var(--bg-deep);
	}
	/* Panel chrome (width/bg/shadow/overflow) comes from Popover via CSS vars */
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.7rem 0.9rem;
		border-bottom: 1px solid var(--border);
	}
	.title {
		font-weight: 650;
		font-size: 0.9rem;
	}
	.markall {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 0.74rem;
		font-weight: 600;
		color: var(--brand);
		background: transparent;
		border: none;
		cursor: pointer;
	}
	.markall:hover {
		text-decoration: underline;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0.3rem;
		max-height: 60vh;
		overflow-y: auto;
	}
	.item {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: start;
		gap: 0.5rem;
		width: 100%;
		padding: 0.6rem 0.6rem;
		border: none;
		background: transparent;
		border-radius: 10px;
		text-align: left;
		cursor: pointer;
		transition: background 0.12s;
	}
	.item:hover {
		background: color-mix(in srgb, var(--text) 7%, transparent);
	}
	.item .dot {
		width: 8px;
		height: 8px;
		margin-top: 5px;
		border-radius: 999px;
		background: var(--brand);
		flex: none;
	}
	.item.unread .msg {
		color: var(--text);
		font-weight: 550;
	}
	.msg {
		grid-column: 2;
		font-size: 0.82rem;
		color: var(--text-2);
		line-height: 1.35;
	}
	/* The dot column may be absent (read items): message still starts at col 2 */
	.item:not(.unread) .msg {
		grid-column: 1 / 3;
	}
	.time {
		font-size: 0.68rem;
		color: var(--text-muted);
		white-space: nowrap;
		margin-top: 2px;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.4rem;
		padding: 2rem 1rem;
		color: var(--text-muted);
		text-align: center;
	}
	.empty p {
		margin: 0;
		font-size: 0.84rem;
	}

	/* --- Toasts (mounted on body via the portal) --- */
	.toasts {
		position: fixed;
		right: 1.1rem;
		bottom: 1.1rem;
		z-index: 60;
		display: flex;
		flex-direction: column-reverse;
		gap: 0.55rem;
		max-width: 360px;
	}
	.toast {
		display: flex;
		align-items: flex-start;
		gap: 0.6rem;
		width: 100%;
		padding: 0.7rem 0.8rem;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		text-align: left;
		cursor: pointer;
		box-shadow: 0 14px 36px -14px rgba(0, 0, 0, 0.6);
	}
	.toast :global(svg:first-child) {
		color: var(--brand);
		flex: none;
		margin-top: 1px;
	}
	.tmsg {
		flex: 1;
		font-size: 0.84rem;
		color: var(--text);
		line-height: 1.35;
	}
	.tclose {
		display: grid;
		place-items: center;
		color: var(--text-muted);
		flex: none;
	}
</style>
