// Calculation and formatting helpers (stateless, easy to test).
// Text labels (daysLabel, cycleLabel…) live in i18n.svelte.ts; here it's only
// pure calculation and money formatting (which uses the active locale).

import { locale } from './i18n.svelte';

export type Lifecycle = { progress: number; days: number };

// progress = fraction of lifetime elapsed (0..1); days = days remaining.
export function lifecycle(startISO: string, dueISO: string, now = new Date()): Lifecycle {
	const start = new Date(startISO).getTime();
	const due = new Date(dueISO).getTime();
	const t = now.getTime();
	const total = due - start || 1;
	const progress = Math.min(1, Math.max(0, (t - start) / total));
	const days = Math.ceil((due - t) / 86_400_000);
	return { progress, days };
}

// Semantic TIME color (not category color).
export function timeColor(progress: number): string {
	if (progress >= 0.85) return 'var(--danger)';
	if (progress >= 0.6) return 'var(--warn)';
	return 'var(--ok)';
}

// Step a YYYY-MM-DD date forward one billing cycle. null for 'once' (no recurrence).
export function advanceCycle(
	dateISO: string,
	cycle: string,
	cycleDays?: number | null
): string | null {
	const months = CYCLE_MONTHS[cycle];
	if (months) return shiftMonths(dateISO, months, 1);
	const d = new Date(dateISO + 'T00:00:00');
	if (cycle === 'custom') d.setDate(d.getDate() + (cycleDays || 30));
	else return null; // once
	return d.toISOString().slice(0, 10);
}

// Recurring cycles expressed in MONTHS, so the date keeps its day-of-month
// instead of drifting on leap years the way a day count does. Mirrors
// `add_cycle` in the backend's scheduler.rs.
export const CYCLE_MONTHS: Record<string, number> = {
	monthly: 1,
	quarterly: 3,
	semiannual: 6,
	yearly: 12,
	biennial: 24
};

// Shift a YYYY-MM-DD date by n months in either direction, clamping to the last
// day of the target month (31 Jan − 1 month = 28/29 Feb), like chrono does.
export function shiftMonths(dateISO: string, months: number, dir: 1 | -1): string | null {
	const d = new Date(dateISO + 'T00:00:00');
	if (isNaN(d.getTime())) return null;
	const day = d.getDate();
	d.setDate(1);
	d.setMonth(d.getMonth() + dir * months);
	const lastDay = new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
	d.setDate(Math.min(day, lastDay));
	return d.toISOString().slice(0, 10);
}

// Short readable date in the active locale (e.g. "1 mar 2027").
export function shortDate(dateISO: string): string {
	const d = new Date(dateISO + 'T00:00:00');
	if (isNaN(d.getTime())) return dateISO;
	return d.toLocaleDateString(locale(), { day: 'numeric', month: 'short', year: 'numeric' });
}

// Average length of a month, for turning an N-day cycle into a monthly figure.
const DAYS_PER_MONTH = 365.25 / 12;

// Cost normalized to one month, in cents (R2: no currency conversion — callers
// group by currency). A named cycle divides by its months; 'custom' scales by
// its day count; 'once' is NOT a recurring cost, so it contributes nothing.
export function monthlyCents(sub: {
	cycle: string;
	amount_cents: number;
	cycle_days?: number | null;
}): number {
	const months = CYCLE_MONTHS[sub.cycle];
	if (months) return sub.amount_cents / months;
	if (sub.cycle === 'custom') {
		const days = sub.cycle_days ?? 30;
		return days > 0 ? (sub.amount_cents * DAYS_PER_MONTH) / days : 0;
	}
	return 0; // 'once': paid a single time, it is not a monthly spend
}

// Does this subscription still cost money going forward? 'paused', 'ended',
// 'cancelled' and 'archived' don't. 'expired' does: the service is still yours
// until you cancel it, you're just late.
export function isOngoing(sub: { status: string }): boolean {
	return sub.status === 'active' || sub.status === 'expired';
}

export function money(cents: number, currency = 'USD'): string {
	const value = cents / 100;
	// ISO code ALWAYS up front (stable across languages and unambiguous between
	// currencies that share the $ symbol). The number uses the language's separators.
	const num = value.toLocaleString(locale(), {
		minimumFractionDigits: 2,
		maximumFractionDigits: 2
	});
	return `${currency} ${num}`;
}
