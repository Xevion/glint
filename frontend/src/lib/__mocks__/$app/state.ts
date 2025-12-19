export const page = $state({
	url: new URL('http://localhost/'),
	params: {},
	route: { id: null },
	status: 200,
	error: null,
	data: {},
	form: undefined,
	state: {}
} as const);
