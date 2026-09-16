import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function ignoreNestedOverlay(e: { target: EventTarget | null; preventDefault: () => void }) {
	const el = e.target instanceof Element ? e.target : null;
	if (
		el?.closest(
			'[data-slot="popover-content"], [data-slot="select-content"], [data-slot="dropdown-menu-content"]'
		)
	) {
		e.preventDefault();
	}
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
