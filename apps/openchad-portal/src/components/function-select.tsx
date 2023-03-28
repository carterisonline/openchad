import { $, component$, useOn } from '@builder.io/qwik';
import { Form } from '@builder.io/qwik-city';
import { JSX } from '@builder.io/qwik/jsx-runtime';
import { EditorProps } from '../routes';

export default component$((props: EditorProps) => {
	useOn(
		'change',
		$((event) => {
			const target = event.target as HTMLInputElement;
			if (target.type === 'radio') {
				props.editorStore.currentFunction = target.value;

				props.editorStore.currentFunctionType =
					target.parentElement?.parentElement?.firstChild?.textContent;
			}
		})
	);

	return (
		<Form>
			<fieldset>
				{radioFieldSetFor(props.config.value.responses, 'Responses')}
				{radioFieldSetFor(props.config.value.macros, 'Macros')}
				{radioFieldSetFor(
					props.config.value.providers,
					'Provider Transformers'
				)}
			</fieldset>
		</Form>
	);
});

function radioFieldSetFor(o: object, label: string): JSX.Element {
	const results = [];
	for (const key of Object.keys(o)) {
		results.push(
			<label for={key}>
				<input type="radio" id={key} name="function" value={key} />
				<code>{key}</code>
			</label>
		);
	}

	return (
		<fieldset>
			<legend>{label}</legend>
			{results}
		</fieldset>
	);
}
