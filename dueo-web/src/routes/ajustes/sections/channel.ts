import type { Component } from 'svelte';
import type { Res } from '$lib/api';

// Telegram and Email are the same form (a destination + an enable toggle + a
// test); ChannelSection renders it once. Each channel supplies this config to
// normalise its status shape, API calls, icon and i18n keys.
export type ChannelConfig = {
	icon: Component; // test-button icon
	inputType: string;
	placeholder: string;
	fieldKey: string;
	hintKey: string;
	enableKey: string;
	warnPreKey: string;
	warnCode: string; // env var shown in the not-configured warning
	testOkKey: string;
	testErrKey: string;
	load: () => Promise<{ ready: boolean; enabled: boolean; value: string } | null>;
	save: (value: string, enabled: boolean) => Promise<Res<unknown>>;
	test: () => Promise<Res<unknown>>;
};
