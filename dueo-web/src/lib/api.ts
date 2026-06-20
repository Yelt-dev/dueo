// Minimal API client. Everything hangs off /api (Vite proxy → Rust backend);
// the session cookie rides along (same origin).

import { goto } from '$app/navigation';

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

// On a 401 from a protected call, the session is gone: redirect to /login once
// so a mid-load expiry can't leave a half-rendered page. Auth probes pass
// redirect=false (a 401 there is expected and handled by the caller).
async function req(path: string, opts: RequestInit = {}, redirect = true) {
	const res = await fetch('/api' + path, {
		headers: { 'Content-Type': 'application/json' },
		...opts
	});
	if (
		res.status === 401 &&
		redirect &&
		typeof window !== 'undefined' &&
		!location.pathname.startsWith('/login')
	) {
		goto('/login');
	}
	return res;
}

export const login = (username: string, password: string) =>
	req('/login', { method: 'POST', body: JSON.stringify({ username, password }) }, false);

export const register = (username: string, password: string) =>
	req('/register', { method: 'POST', body: JSON.stringify({ username, password }) }, false);

export const getSetupStatus = () => req('/setup-status', {}, false);

export const logout = () => req('/logout', { method: 'POST' });

export const me = () => req('/me', {}, false);

export const updateSettings = (body: {
	timezone?: string;
	send_hour?: number;
	default_currency?: string;
	lang?: string;
}) => req('/settings', { method: 'PUT', body: JSON.stringify(body) });

// ---- Seguridad ------------------------------------------------------------

export const changePassword = (body: { current_password: string; new_password: string }) =>
	req('/password', { method: 'POST', body: JSON.stringify(body) });

export const logoutAll = () => req('/logout-all', { method: 'POST' });

// ---- Datos (export / import) ----------------------------------------------

export const exportData = () => req('/export');
export const importData = (backup: unknown) =>
	req('/import', { method: 'POST', body: JSON.stringify(backup) });

// ---- Usuarios (solo admin) -------------------------------------------------

export type AdminUser = {
	id: number;
	username: string;
	role: string;
	created_at: string;
};

export const getUsers = () => req('/users');
export const createUser = (body: { username: string; password: string; role?: string }) =>
	req('/users', { method: 'POST', body: JSON.stringify(body) });
export const deleteUser = (id: number) => req(`/users/${id}`, { method: 'DELETE' });

// ---- Acerca de / salud -----------------------------------------------------

export type VersionInfo = { name: string; version: string };

export const getVersion = () => req('/version');
export const getHealth = () => req('/health');

export const getSubscriptions = () => req('/subscriptions');

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

export const createSubscription = (body: NewSub) =>
	req('/subscriptions', { method: 'POST', body: JSON.stringify(body) });

export const updateSubscription = (id: number, body: Partial<NewSub> & Record<string, unknown>) =>
	req(`/subscriptions/${id}`, { method: 'PATCH', body: JSON.stringify(body) });

export const deleteSubscription = (id: number) => req(`/subscriptions/${id}`, { method: 'DELETE' });

// ---- Categorías -----------------------------------------------------------

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

export const getCategories = () => req('/categories');

// ---- Notificaciones (panel in-app) ----------------------------------------

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

export const getNotifications = () => req('/notifications');
export const markNotificationRead = (id: number) =>
	req(`/notifications/${id}/read`, { method: 'POST' });
export const markAllNotificationsRead = () => req('/notifications/read', { method: 'POST' });

// ---- Recordatorios (reglas de anticipación) --------------------------------

export type Reminder = {
	id: number;
	subscription_id: number | null; // null = regla global del usuario
	days_before: number;
};

export const getReminders = () => req('/reminders');
export const createReminder = (body: { subscription_id: number | null; days_before: number }) =>
	req('/reminders', { method: 'POST', body: JSON.stringify(body) });
export const deleteReminder = (id: number) => req(`/reminders/${id}`, { method: 'DELETE' });

// ---- Telegram (canal) ------------------------------------------------------

export type TelegramStatus = {
	bot_ready: boolean; // ¿el servidor tiene token?
	enabled: boolean;
	chat_id: string | null;
};

export const getTelegramStatus = () => req('/channels/telegram');
export const setTelegram = (body: { chat_id: string; enabled?: boolean }) =>
	req('/channels/telegram', { method: 'PUT', body: JSON.stringify(body) });
export const testTelegram = () => req('/channels/telegram/test', { method: 'POST' });

// ---- Email (canal) ---------------------------------------------------------

export type EmailStatus = {
	smtp_ready: boolean; // ¿el servidor tiene SMTP?
	enabled: boolean;
	email: string | null;
};

export const getEmailStatus = () => req('/channels/email');
export const setEmail = (body: { email: string; enabled?: boolean }) =>
	req('/channels/email', { method: 'PUT', body: JSON.stringify(body) });
export const testEmail = () => req('/channels/email/test', { method: 'POST' });

export const createCategory = (body: NewCategory) =>
	req('/categories', { method: 'POST', body: JSON.stringify(body) });

export const updateCategory = (id: number, body: Partial<NewCategory>) =>
	req(`/categories/${id}`, { method: 'PATCH', body: JSON.stringify(body) });

export const deleteCategory = (id: number) => req(`/categories/${id}`, { method: 'DELETE' });
