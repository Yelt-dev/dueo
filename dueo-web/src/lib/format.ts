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
	const d = new Date(dateISO + 'T00:00:00');
	if (cycle === 'monthly') d.setMonth(d.getMonth() + 1);
	else if (cycle === 'yearly') d.setFullYear(d.getFullYear() + 1);
	else if (cycle === 'custom') d.setDate(d.getDate() + (cycleDays || 30));
	else return null; // once
	return d.toISOString().slice(0, 10);
}

// Cost normalized to one month, in cents (R2: no currency conversion — callers
// group by currency). Yearly is /12; other cycles count as-is.
export function monthlyCents(sub: { cycle: string; amount_cents: number }): number {
	return sub.cycle === 'yearly' ? sub.amount_cents / 12 : sub.amount_cents;
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
