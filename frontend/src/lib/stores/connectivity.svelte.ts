import { browser } from '$app/environment';

let online = $state(browser ? navigator.onLine : true);
let backendReachable = $state(true);

if (browser) {
	window.addEventListener('online', () => (online = true));
	window.addEventListener('offline', () => (online = false));
}

export const connectivity = {
	get online() {
		return online;
	},
	get backendReachable() {
		return backendReachable;
	},
	get hasIssue() {
		return !online || !backendReachable;
	},
	reportBackendSuccess() {
		backendReachable = true;
	},
	reportBackendError() {
		backendReachable = false;
	}
};
