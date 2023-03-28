import { component$, $, useStyles$, useOnWindow } from '@builder.io/qwik';
import styles from './text-area-grow.css?inline';

export default component$((props: { name: string; value: string }) => {
	useStyles$(styles);

	const resize = $((event: Event) => {
		const target = event.target as HTMLInputElement;
		target.style.height = `${target.scrollHeight}px`;
	});

	useOnWindow('load', resize);

	return (
		<label for={props.name}>
			{props.name}
			<textarea
				id={props.name}
				name={props.name}
				value={props.value}
				onInput$={resize}
				onLoadCapture$={resize}
				spellcheck={false}
			/>
		</label>
	);
});
