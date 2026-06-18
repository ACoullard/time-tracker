import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { Time } from "@internationalized/date";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

export function formatElapsed(ms: number, truncateSeconds = false): string {
	const total = Math.max(0, Math.floor(ms / 1000));
	const h = Math.floor(total / 3600);
	const m = Math.floor((total % 3600) / 60);
	const s = total % 60;
	if (truncateSeconds) return `${h}:${m.toString().padStart(2, "0")}`;
	return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export function msToTime(ms: number): Time {
	const d = new Date(ms);
	return new Time(d.getHours(), d.getMinutes());
}

export function applyTimeToMs(originalMs: number, t: Time): number {
	const d = new Date(originalMs);
	d.setHours(t.hour, t.minute, 0, 0);
	return d.getTime();
}

export function formatIsoYMD(d: Date): string {
	return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

export function formatTime(ms: number): string {
	return new Date(ms).toLocaleTimeString(undefined, {
		hour: "numeric",
		minute: "2-digit",
		hour12: true,
	});
}
