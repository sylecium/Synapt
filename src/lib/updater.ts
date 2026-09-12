import { check, type Update } from '@tauri-apps/plugin-updater';

export async function probeUpdate(): Promise<Update | null> {
	if (import.meta.env.DEV) return null;
	try {
		const update = await check();
		return update ?? null;
	} catch {
		return null;
	}
}
