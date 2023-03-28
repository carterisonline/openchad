import {
	component$,
	Resource,
	Signal,
	useResource$,
	useSignal,
	$,
	noSerialize,
	useStyles$,
} from '@builder.io/qwik';
import { editor, languages } from 'monaco-editor';
import dobriChad from '../types/Dobri-A03.json';
import { conf, language } from '../types/jinja2';
import styles from './monaco-editor.css?inline';

export default component$(
	(props: {
		content: Readonly<Signal<string>>;
		editorLabel: string;
		maxNum: number;
		index: number;
	}) => {
		useStyles$(styles);

		const monacoSignal = useSignal<any>(noSerialize([]));
		const monacoResource = useResource$(async (ctx) => {
			ctx.track(() => props.content.value);

			const monaco = await import('monaco-editor');

			if (monaco.editor.getModels().length >= props.maxNum) {
				monaco.editor.getModels().forEach((model) => model.dispose());
			}

			if (monaco.editor.getModels().length === 0) {
				monaco.languages.register({ id: 'jinja2' });
				monaco.languages.setLanguageConfiguration('jinja2', conf);
				monaco.languages.setMonarchTokensProvider('jinja2', language);

				monaco.languages.registerCompletionItemProvider('jinja2', {
					provideCompletionItems: () => {
						const suggestions = [
							{
								label: 'simpleText',
								kind: monaco.languages.CompletionItemKind.Text,
								insertText: 'simpleText',
							},
						];
						return {
							suggestions,
						} as languages.CompletionList;
					},
				});

				monaco.editor.defineTheme(
					'dobri-chad',
					dobriChad as editor.IStandaloneThemeData
				);
			}

			const container = document.getElementById(
				`${props.editorLabel}-editor`
			);

			if (!container) {
				throw new Error(`#${props.editorLabel}-editor not found`);
			}

			monaco.editor.create(container, {
				ariaLabel: props.editorLabel,
				value: props.content.value,
				language: 'jinja2',
				minimap: {
					enabled: false,
				},
				lineNumbers: 'off',
				scrollbar: {
					vertical: 'hidden',
				},
				theme: 'dobri-chad',
				padding: {
					top: 10,
				},
			});

			monacoSignal.value.push(monaco);
			container.style.visibility = 'visible';
			// } else {
			// 	monaco.editor.getModels()[0].setValue(props.content.value);
			// }
		});

		const fixHeight = $(() => {
			const editor = monacoSignal.value[0].editor.getEditors()[
				props.index
			] as editor.ICodeEditor;
			const model = monacoSignal.value[0].editor.getModels()[
				props.index
			] as editor.ITextModel;

			const contentHeight = model.getLineCount() * 20 + 40;

			const container = document.getElementById(
				`${props.editorLabel}-editor`
			);

			if (!container) {
				throw new Error(`Couldn't find #${props.editorLabel}-editor`);
			}

			container.style.height = `${contentHeight + 10}px`;

			editor.layout({
				width: Math.min(550, editor.getContentWidth()),
				height: contentHeight,
			});
		});

		return (
			<div>
				<label for={`${props.editorLabel}-editor`}>
					{props.editorLabel.charAt(0).toUpperCase() +
						props.editorLabel.slice(1)}
					<Resource
						value={monacoResource}
						onPending={() => (
							<textarea placeholder="Loading Monaco..."></textarea>
						)}
						onRejected={() => (
							<div>Failed to load Monaco Editor</div>
						)}
						onResolved={() => {
							return <div />;
						}}
					/>
					<section
						id={`${props.editorLabel}-editor`}
						onKeyDown$={fixHeight}
						onSubmit$={fixHeight}
					/>
				</label>
			</div>
		);
	}
);
