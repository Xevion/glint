import { describe, it, expect } from 'vitest';
import { mount } from 'svelte';
import Button from './button.svelte';

describe('Button component', () => {
	it('renders a button element', () => {
		const target = document.createElement('div');
		document.body.appendChild(target);

		mount(Button, { target });

		const button = target.querySelector('button');
		expect(button).toBeTruthy();

		document.body.removeChild(target);
	});

	it('renders with href as anchor element', () => {
		const target = document.createElement('div');
		document.body.appendChild(target);

		mount(Button, { target, props: { href: '/test' } });

		const anchor = target.querySelector('a');
		expect(anchor).toBeTruthy();
		expect(anchor?.getAttribute('href')).toBe('/test');

		document.body.removeChild(target);
	});
});
