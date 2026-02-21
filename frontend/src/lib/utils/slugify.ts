/** Convert a name to a URL-safe slug (mirrors backend slugify). */
export function slugify(name: string): string {
	let slug = '';
	let prevHyphen = true;
	for (const c of name) {
		if (/[a-zA-Z0-9]/.test(c)) {
			slug += c.toLowerCase();
			prevHyphen = false;
		} else if (c === "'") {
			// Strip apostrophes (possessives: "Sildur's" -> "sildurs")
		} else if (!prevHyphen) {
			slug += '-';
			prevHyphen = true;
		}
	}
	return slug.endsWith('-') ? slug.slice(0, -1) : slug;
}
