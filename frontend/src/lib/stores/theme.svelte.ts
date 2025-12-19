import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

function getInitialTheme(): Theme {
	if (!browser) return 'light';
	// Read from DOM - the blocking script in app.html already set this
	return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
}

function createThemeStore() {
	let theme = $state<Theme>(getInitialTheme());

	function init() {
		if (!browser) return;

		// Listen for system preference changes (only when no explicit preference is stored)
		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
			if (!localStorage.getItem('theme')) {
				theme = e.matches ? 'dark' : 'light';
				applyTheme();
			}
		});
	}

	function applyTheme() {
		if (!browser) return;
		document.documentElement.classList.toggle('dark', theme === 'dark');
	}

	function toggle() {
		theme = theme === 'dark' ? 'light' : 'dark';
		if (browser) {
			localStorage.setItem('theme', theme);
		}
		applyTheme();
	}

	function set(newTheme: Theme) {
		theme = newTheme;
		if (browser) {
			localStorage.setItem('theme', theme);
		}
		applyTheme();
	}

	return {
		get current() {
			return theme;
		},
		get isDark() {
			return theme === 'dark';
		},
		init,
		toggle,
		set
	};
}

export const themeStore = createThemeStore();
