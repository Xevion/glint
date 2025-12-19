import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

function createThemeStore() {
	let theme = $state<Theme>('light');

	function init() {
		if (!browser) return;

		// Check localStorage first, then system preference
		const stored = localStorage.getItem('theme') as Theme | null;
		if (stored === 'light' || stored === 'dark') {
			theme = stored;
		} else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
			theme = 'dark';
		}

		applyTheme();

		// Listen for system preference changes
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
