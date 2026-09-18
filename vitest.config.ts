import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	test: {
		include: ['tests/components/**/*.ui.ts'],
		environment: 'happy-dom',
		setupFiles: ['./tests/setup-dom.ts'],
		globals: true
	},
	resolve: {
		conditions: ['browser']
	}
});
