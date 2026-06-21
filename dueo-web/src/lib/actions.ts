// Shared Svelte actions.

// Reparent a node to <body>. Needed for `position: fixed` overlays that live
// inside an ancestor with `backdrop-filter` (e.g. the topbar), which otherwise
// becomes the containing block and breaks viewport-relative positioning.
export function portal(node: HTMLElement) {
	document.body.appendChild(node);
	return { destroy: () => node.remove() };
}
