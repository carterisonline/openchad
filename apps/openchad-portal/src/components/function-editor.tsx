import { component$, Signal, useSignal, useTask$ } from '@builder.io/qwik';
import { Form } from '@builder.io/qwik-city';
import { EditorProps } from '../routes';
import { ConfigResponse } from '../types/bot-config';
import MonacoEditor from './monaco-editor';
import TextAreaGrow from './text-area-grow';

export const Inner = component$((props: EditorProps) => {
	console.info(props.editorStore.currentFunction);
	if (!props.editorStore.currentFunction) {
		return noFunctionSelected();
	}

	const functionType =
		props.editorStore.currentFunctionType ??
		(() => {
			console.error('No function type selected');
			('Responses');
		})();

	switch (functionType) {
		case 'Responses':
			return (
				<ResponseForm
					functionIdent={
						props.editorStore.currentFunction ?? 'unknown'
					}
					functionConfig={
						props.config.value.responses[
							props.editorStore.currentFunction ?? 'unknown'
						]
					}
				></ResponseForm>
			);
		case 'Macros':
			return <div> macros </div>;
		case 'Provider Transformers':
			return <div> providers </div>;
		default:
			console.error('No function type selected');
			return <div> failed to render </div>;
	}
});

export default component$((props: EditorProps) => {
	return <Inner {...props} />;
});

export const ResponseForm = component$(
	(props: { functionIdent: string; functionConfig: ConfigResponse }) => {
		const prompt = useSignal('');
		const footer = useSignal(props.functionConfig.footer ?? '');

		useTask$(({ track }) => {
			const promptArray = track(() => props.functionConfig.prompt);
			prompt.value = promptArray.join('\n');
		});

		return (
			<Form>
				<FunctionIdent currentFunction={props.functionIdent} />
				<MonacoEditor
					content={prompt}
					editorLabel="prompt"
					maxNum={props.functionConfig.footer ? 2 : 1}
					index={0}
				/>
				{props.functionConfig.footer && (
					<MonacoEditor
						content={footer}
						editorLabel="footer"
						maxNum={2}
						index={1}
					/>
				)}
			</Form>
		);
	}
);

export const FunctionIdent = component$(
	(props: { currentFunction: string }) => {
		return (
			<label for="functionIdent">
				Function Identifier
				<input
					type="text"
					id="functionIdent"
					name="functionIdent"
					value={props.currentFunction}
				/>
			</label>
		);
	}
);

export function noFunctionSelected() {
	// TypeScript doesn't like align="center" so I'll use HTML4 :sunglasses:
	return (
		<center>
			<h1>No Function Selected</h1>
			<p>Select a function using the sidebar on the left.</p>
		</center>
	);
}
