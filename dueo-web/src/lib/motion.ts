// Staggered page entrance (STANDARD, see doc context/10): a page's blocks appear
// one after another from the top instead of the whole view popping in at once.
//
// Pass the block's visual order (0, 1, 2…). Rows inside a list keep counting
// from where their section left off, so the cascade never restarts mid-page.
// Feed the result straight to `in:fly`.

export const ENTER_STEP = 60; // ms between consecutive blocks
export const ENTER_MS = 280; // how long each block takes
export const ENTER_Y = 12; // px it travels upward

function reducedMotion(): boolean {
	if (typeof window === 'undefined') return false;
	return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
}

export function enter(order = 0, opts: { y?: number; duration?: number } = {}) {
	// Honour the system setting: same layout, no movement.
	if (reducedMotion()) return { y: 0, duration: 0, delay: 0 };
	return {
		y: opts.y ?? ENTER_Y,
		duration: opts.duration ?? ENTER_MS,
		delay: order * ENTER_STEP
	};
}

// Same cascade, but tied to the viewport: a row that enters below the fold
// animates when you scroll to it, not while nobody is looking. `in:fly` fires
// on mount, so long lists played their entrance off-screen and looked static
// by the time you got there.
//
// Pass the same `order` as `enter()`. Rows visible at mount keep the stagger;
// rows reached by scrolling appear at once, because the scroll IS the stagger.
export function reveal(node: HTMLElement, order = 0) {
	if (reducedMotion() || typeof IntersectionObserver === 'undefined') return;

	const born = performance.now();
	node.style.opacity = '0';
	node.style.transform = `translateY(${ENTER_Y}px)`;

	const show = (scrolledTo: boolean) => {
		const delay = scrolledTo ? 0 : order * ENTER_STEP;
		node.style.transition = `opacity ${ENTER_MS}ms ease, transform ${ENTER_MS}ms ease`;
		node.style.transitionDelay = `${delay}ms`;
		node.style.opacity = '1';
		node.style.transform = 'translateY(0)';
		// Drop every inline style once it lands, so `animate:flip` and the row's
		// own hover transform get their transform back.
		setTimeout(
			() => {
				node.style.transition = '';
				node.style.transitionDelay = '';
				node.style.transform = '';
				node.style.opacity = '';
			},
			ENTER_MS + delay + 60
		);
	};

	const io = new IntersectionObserver(
		(entries) => {
			if (!entries.some((e) => e.isIntersecting)) return;
			io.disconnect();
			show(performance.now() - born > 400);
		},
		{ rootMargin: '0px 0px -8% 0px' }
	);
	io.observe(node);

	return { destroy: () => io.disconnect() };
}
