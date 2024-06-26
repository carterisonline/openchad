import { qwikVite } from '@builder.io/qwik/optimizer';
import { qwikCity } from '@builder.io/qwik-city/vite';
import { defineConfig } from 'vite';
import tsconfigPaths from 'vite-tsconfig-paths';
import { qwikNxVite } from 'qwik-nx/plugins';
import monacoEditorPlugin from 'vite-plugin-monaco-editor';
import * as fs from 'node:fs';

/** @type {import('vite').Plugin} */
const bufferLoader = {
	name: 'hex-loader',
	async transform(code: string, id: string) {
		const [path, query] = id.split('?');
		if (query != 'raw') return null;

		const fileContents = fs.readFileSync(path);
		const array = new Uint8Array(fileContents).join(',');

		return `export default new Uint8Array([${array}]).buffer`;
	},
};

export default defineConfig({
	cacheDir: '../../node_modules/.vite/apps/openchad-portal',
	plugins: [
		qwikNxVite(),
		qwikCity(),
		qwikVite({
			client: {
				outDir: '../../dist/apps/openchad-portal/client',
			},
			ssr: {
				outDir: '../../dist/apps/openchad-portal/server',
			},
		}),
		tsconfigPaths({ root: '../../' }),
		monacoEditorPlugin({}),
		bufferLoader,
	],
	server: {
		fs: {
			// Allow serving files from the project root
			allow: ['../../'],
		},
	},
	preview: {
		headers: {
			'Cache-Control': 'public, max-age=600',
		},
	},
	test: {
		globals: true,
		cache: {
			dir: '../../node_modules/.vitest',
		},
		environment: 'node',
		include: ['src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
	},
});
