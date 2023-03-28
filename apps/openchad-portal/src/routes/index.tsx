import { component$, Signal, useStore } from '@builder.io/qwik';
import { DocumentHead, routeLoader$ } from '@builder.io/qwik-city';
import FunctionEditor from '../components/function-editor';
import FunctionSelect from '../components/function-select';
import { BotConfig } from '../types/bot-config';

export const useGetConfig = routeLoader$(async () => {
	const apiUrl = process.env['API_URL'];
	if (!apiUrl) {
		throw new Error('API_URL not set');
	}

	const url = `http://${apiUrl}/config`;
	try {
		const response = await fetch(url);
		const config: BotConfig = await response.json();
		return config;
	} catch (e) {
		throw new Error(`Failed to fetch config: ${e}`);
	}
});

export interface EditorStore {
	currentFunction: string | null;
	currentFunctionType: string | null | undefined;
}

export interface EditorProps {
	editorStore: EditorStore;
	config: Readonly<Signal<BotConfig>>;
}

export default component$(() => {
	try {
		const config = useGetConfig();

		const editorStore: EditorStore = useStore({
			currentFunction: null,
			currentFunctionType: null,
		});

		return (
			<section class="grid">
				<aside>
					<h2>Select a Function</h2>
					<FunctionSelect editorStore={editorStore} config={config} />
				</aside>
				<FunctionEditor editorStore={editorStore} config={config} />
			</section>
		);
	} catch (e) {
		console.error(e);
		return (
			<section class="grid">
				<aside role="alert">Failed to load config</aside>
			</section>
		);
	}
});

export const head: DocumentHead = {
	title: 'OpenChad Configuration',
	meta: [
		{
			name: 'description',
			content: 'Configure the OpenChad API',
		},
	],
};
