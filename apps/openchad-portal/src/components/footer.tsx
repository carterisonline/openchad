import { component$ } from '@builder.io/qwik';

export default component$(() => {
	return (
		<footer role="contentinfo" class="container">
			<strong>Copyright © {new Date().getFullYear()} Carter Reeb.</strong>
			<address>
				This tool is free software licensed under the{' '}
				<a href="https://www.apache.org/licenses/LICENSE-2.0">
					Apache 2.0 License
				</a>
				.
			</address>
		</footer>
	);
});
