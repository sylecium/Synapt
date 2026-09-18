import '@testing-library/jest-dom/vitest';

if (typeof window !== 'undefined') {
	if (!globalThis.ResizeObserver) {
		globalThis.ResizeObserver = class ResizeObserver {
			observe() {}
			unobserve() {}
			disconnect() {}
		};
	}

	if (!window.Element.prototype.scrollIntoView) {
		window.Element.prototype.scrollIntoView = () => {};
	}

	if (!window.PointerEvent) {
		class PointerEvent extends MouseEvent {
			pointerId: number;
			constructor(type: string, params: PointerEventInit = {}) {
				super(type, params);
				this.pointerId = params.pointerId ?? 0;
			}
		}
		// @ts-expect-error polyfill
		window.PointerEvent = PointerEvent;
		// @ts-expect-error polyfill
		globalThis.PointerEvent = PointerEvent;
	}
}
