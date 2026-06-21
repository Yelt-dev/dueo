// Minimal API client. Everything hangs off /api (Vite proxy → Rust backend);
// the session cookie rides along (same origin).

import { goto } from '$app/navigation';

// Every call resolves to this shape — callers branch on `ok`/`status` and read
// `data` (parsed JSON on success) or `error` (raw body text on failure). The
// request never throws on a non-2xx; only a network failure rejects.
export type Res<T> = {
	ok: boolean;
	status: number;
	data: T | null;
	error: string | null;
};

// On a 401 from a protected call, the session is gone: redirect to /login once
// so a mid-load expiry can't leave a half-rendered page. Auth probes pass
// redirect=false (a 401 there is expected and handled by the caller).
async function req<T = unknown>(
	path: string,
	opts: RequestInit = {},
	redirect = true
): Promise<Res<T>> {
	// Only declare JSON when we actually send a body; bodiless GET/DELETE/POST
	// shouldn't advertise a content type they don't have.
	const headers =
		opts.body != null ? { 'Content-Type': 'application/json', ...opts.headers } : opts.headers;
	const res = await fetch('/api' + path, { ...opts, headers });
	if (
		res.status === 401 &&
		redirect &&
		typeof window !== 'undefined' &&
		!location.pathname.startsWith('/login')
	) {
		goto('/login');
	}
	const text = await res.text();
	if (!res.ok) return { ok: false, status: res.status, data: null, error: text || null };
	// Empty body (logout, delete, …) is a valid success with no data.
	let data: T | null = null;
	if (text) {
		try {
			data = JSON.parse(text) as T;
		} catch {
			data = null;
		}
	}
	return { ok: true, status: res.status, data, error: null };
}

// ---- Auth / session --------------------------------------------------------

export type User = {
	id: number;
	username: string;
	role: string;
	timezone: string;
	send_hour: number;
	default_currency: string;
	lang: string;
};

export type SetupStatus = {
	needs_setup: boolean;
	open_registration: boolean;
};

export const login = (username: string, password: string) =>
	req<User>('/login', { method: 'POST', body: JSON.stringify({ username, password }) }, false);

export const register = (username: string, password: string) =>
	req<User>('/register', { method: 'POST', body: JSON.stringify({ username, password }) }, false);

export const getSetupStatus = () => req<SetupStatus>('/setup-status', {}, false);

export const logout = () => req('/logout', { method: 'POST' });

export const me = () => req<User>('/me', {}, false);

export const updateSettings = (body: {
	timezone?: string;
	send_hour?: number;
	default_currency?: string;
	lang?: string;
}) => req<User>('/settings', { method: 'PUT', body: JSON.stringify(body) });

// ---- Security -------------------------------------------------------------

export const changePassword = (body: { current_password: string; new_password: string }) =>
	req('/password', { method: 'POST', body: JSON.stringify(body) });

export const logoutAll = () => req('/logout-all', { method: 'POST' });

// ---- Data (export / import) -----------------------------------------------

export type ImportResult = { categories: number; subscriptions: number; reminders: number };

export const exportData = () => req<unknown>('/export');
export const importData = (backup: unknown) =>
	req<ImportResult>('/import', { method: 'POST', body: JSON.stringify(backup) });

// ---- Users (admin only) ----------------------------------------------------

export type AdminUser = {
	id: number;
	username: string;
	role: string;
	created_at: string;
};

export const getUsers = () => req<AdminUser[]>('/users');
export const createUser = (body: { username: string; password: string; role?: string }) =>
	req<AdminUser>('/users', { method: 'POST', body: JSON.stringify(body) });
export const deleteUser = (id: number) => req(`/users/${id}`, { method: 'DELETE' });

// ---- About / health --------------------------------------------------------

export type VersionInfo = { name: string; version: string };

export const getVersion = () => req<VersionInfo>('/version');
export const getHealth = () => req<string>('/health');

// ---- Subscriptions ---------------------------------------------------------

export type Sub = {
	id: number;
	name: string;
	amount_cents: number;
	currency: string;
	cycle: string;
	cycle_days: number | null;
	start_date: string;
	due_date: string;
	category_id: number | null;
	payment_mode: string;
	status: string;
	notes: string | null;
	icon: string | null;
	color: string | null;
};

export type NewSub = {
	name: string;
	amount_cents: number;
	currency: string;
	cycle: string;
	cycle_days?: number | null;
	start_date: string;
	due_date: string;
	category_id?: number | null;
	payment_mode: string;
	notes?: string | null;
	icon?: string | null;
	color?: string | null;
};

// A PATCH only carries the fields being changed; `status` is patchable here
// (it isn't part of NewSub, which is creation-only).
export type UpdateSub = Partial<NewSub> & { status?: string };

export const getSubscriptions = () => req<Sub[]>('/subscriptions');

export const createSubscription = (body: NewSub) =>
	req<Sub>('/subscriptions', { method: 'POST', body: JSON.stringify(body) });

export const updateSubscription = (id: number, body: UpdateSub) =>
	req<Sub>(`/subscriptions/${id}`, { method: 'PATCH', body: JSON.stringify(body) });

export const deleteSubscription = (id: number) => req(`/subscriptions/${id}`, { method: 'DELETE' });

// ---- Categories -----------------------------------------------------------

export type Category = {
	id: number;
	name: string;
	color: string | null;
	icon: string | null;
	sort_order: number;
};

export type NewCategory = {
	name: string;
	color?: string | null;
	icon?: string | null;
	sort_order?: number;
};

export const getCategories = () => req<Category[]>('/categories');
export const createCategory = (body: NewCategory) =>
	req<Category>('/categories', { method: 'POST', body: JSON.stringify(body) });
export const updateCategory = (id: number, body: Partial<NewCategory>) =>
	req<Category>(`/categories/${id}`, { method: 'PATCH', body: JSON.stringify(body) });
export const deleteCategory = (id: number) => req(`/categories/${id}`, { method: 'DELETE' });

// ---- Notifications (in-app panel) -----------------------------------------

export type Notification = {
	id: number;
	subscription_id: number;
	channel: string;
	target_due_date: string;
	days_before: number;
	message: string;
	created_at: string;
	read_at: string | null;
};

export const getNotifications = () => req<Notification[]>('/notifications');
export const markNotificationRead = (id: number) =>
	req(`/notifications/${id}/read`, { method: 'POST' });
export const markAllNotificationsRead = () => req('/notifications/read', { method: 'POST' });

// ---- Reminders (lead-time rules) -------------------------------------------

export type Reminder = {
	id: number;
	subscription_id: number | null; // null = user's global rule
	days_before: number;
};

export const getReminders = () => req<Reminder[]>('/reminders');
export const createReminder = (body: { subscription_id: number | null; days_before: number }) =>
	req<Reminder>('/reminders', { method: 'POST', body: JSON.stringify(body) });
export const deleteReminder = (id: number) => req(`/reminders/${id}`, { method: 'DELETE' });

// ---- Telegram (channel) ----------------------------------------------------

export type TelegramStatus = {
	bot_ready: boolean; // does the server have a token?
	enabled: boolean;
	chat_id: string | null;
};

export const getTelegramStatus = () => req<TelegramStatus>('/channels/telegram');
export const setTelegram = (body: { chat_id: string; enabled?: boolean }) =>
	req('/channels/telegram', { method: 'PUT', body: JSON.stringify(body) });
export const testTelegram = () => req('/channels/telegram/test', { method: 'POST' });

// ---- Email (channel) -------------------------------------------------------

export type EmailStatus = {
	smtp_ready: boolean; // does the server have SMTP?
	enabled: boolean;
	email: string | null;
};

export const getEmailStatus = () => req<EmailStatus>('/channels/email');
export const setEmail = (body: { email: string; enabled?: boolean }) =>
	req('/channels/email', { method: 'PUT', body: JSON.stringify(body) });
export const testEmail = () => req('/channels/email/test', { method: 'POST' });
