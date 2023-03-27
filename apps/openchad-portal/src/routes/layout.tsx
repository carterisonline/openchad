import { component$, Slot } from '@builder.io/qwik';
import Header from '../components/header/nav';

export default component$(() => {
	return (
		<>
			<main>
				<Header />
				<section>
					<Slot />
				</section>
			</main>
			<footer>
				<a href="https://www.builder.io/" target="_blank">
					Made with ♡ by Builder.io
				</a>
			</footer>
		</>
	);
});
