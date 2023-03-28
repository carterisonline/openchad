import { component$ } from '@builder.io/qwik';

export default component$(() => {
	return (
		<header role="banner" class="container">
			<nav>
				<ul>
					<li>
						<strong>OpenChad Configuration</strong>
					</li>
				</ul>
				<ul>
					<li>
						<a
							href="https://qwik.builder.io/docs/components/overview/"
							target="_blank"
						>
							Docs
						</a>
					</li>
					<li>
						<a
							href="https://qwik.builder.io/examples/introduction/hello-world/"
							target="_blank"
						>
							Examples
						</a>
					</li>
					<li>
						<a
							href="https://qwik.builder.io/tutorial/welcome/overview/"
							target="_blank"
						>
							Tutorials
						</a>
					</li>
				</ul>
			</nav>
		</header>
	);
});
