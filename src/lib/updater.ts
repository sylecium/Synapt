import { check, type Update } from '@tauri-apps/plugin-updater';

export const UPDATE_CHECK_INTERVAL_MS = 15 * 60 * 1000;

export async function probeUpdate(): Promise<Update | null> {
	if (import.meta.env.DEV) return null;
	try {
		const update = await check();
		return update ?? null;
	} catch {
		return null;
	}
}
