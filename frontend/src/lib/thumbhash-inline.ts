/**
 * Self-contained thumbhash decode logic for injection as an inline <script> in app.html.
 *
 * Runs before SvelteKit hydration to decode `data-thumbhash` attributes into
 * CSS background-image data URLs, ensuring placeholders are visible on first paint.
 *
 * Uses the mode-watcher pattern: the function is serialized via `.toString()` and
 * injected into the HTML via `transformPageChunk` in hooks.server.ts.
 *
 * The decode functions are copied verbatim from evanw/thumbhash (MIT).
 * They must remain self-contained — no imports allowed since this runs as an inline script.
 */

// biome-ignore lint/complexity/noExcessiveCognitiveComplexity: vendored algorithm, intentionally self-contained
function initThumbhashes(): void {
	// Begin vendored thumbhash decode (MIT, github.com/evanw/thumbhash)

	function thumbHashToApproximateAspectRatio(hash: Uint8Array): number {
		const header = hash[3];
		const hasAlpha = hash[2] & 0x80;
		const isLandscape = hash[4] & 0x80;
		const lx = isLandscape ? (hasAlpha ? 5 : 7) : header & 7;
		const ly = isLandscape ? header & 7 : hasAlpha ? 5 : 7;
		return lx / ly;
	}

	function thumbHashToRGBA(hash: Uint8Array): { w: number; h: number; rgba: Uint8Array } {
		const { PI, min, max, cos, round } = Math;

		const header24 = hash[0] | (hash[1] << 8) | (hash[2] << 16);
		const header16 = hash[3] | (hash[4] << 8);
		const l_dc = (header24 & 63) / 63;
		const p_dc = ((header24 >> 6) & 63) / 31.5 - 1;
		const q_dc = ((header24 >> 12) & 63) / 31.5 - 1;
		const l_scale = ((header24 >> 18) & 31) / 31;
		const hasAlpha = header24 >> 23;
		const p_scale = ((header16 >> 3) & 63) / 63;
		const q_scale = ((header16 >> 9) & 63) / 63;
		const isLandscape = header16 >> 15;
		const lx = max(3, isLandscape ? (hasAlpha ? 5 : 7) : header16 & 7);
		const ly = max(3, isLandscape ? header16 & 7 : hasAlpha ? 5 : 7);
		const a_dc = hasAlpha ? (hash[5] & 15) / 15 : 1;
		const a_scale = (hash[5] >> 4) / 15;

		const ac_start = hasAlpha ? 6 : 5;
		let ac_index = 0;
		const decodeChannel = (nx: number, ny: number, scale: number): number[] => {
			const ac: number[] = [];
			for (let cy = 0; cy < ny; cy++)
				for (let cx = cy ? 0 : 1; cx * ny < nx * (ny - cy); cx++)
					ac.push(
						(((hash[ac_start + (ac_index >> 1)] >> ((ac_index++ & 1) << 2)) & 15) / 7.5 - 1) * scale
					);
			return ac;
		};
		const l_ac = decodeChannel(lx, ly, l_scale);
		const p_ac = decodeChannel(3, 3, p_scale * 1.25);
		const q_ac = decodeChannel(3, 3, q_scale * 1.25);
		const a_ac = hasAlpha ? decodeChannel(5, 5, a_scale) : null;

		const ratio = thumbHashToApproximateAspectRatio(hash);
		const w = round(ratio > 1 ? 32 : 32 * ratio);
		const h = round(ratio > 1 ? 32 / ratio : 32);
		const rgba = new Uint8Array(w * h * 4);
		const fx: number[] = [];
		const fy: number[] = [];
		for (let y = 0, i = 0; y < h; y++) {
			for (let x = 0; x < w; x++, i += 4) {
				let l = l_dc;
				let p = p_dc;
				let q = q_dc;
				let a = a_dc;
				for (let cx = 0, n = max(lx, hasAlpha ? 5 : 3); cx < n; cx++)
					fx[cx] = cos((PI / w) * (x + 0.5) * cx);
				for (let cy = 0, n = max(ly, hasAlpha ? 5 : 3); cy < n; cy++)
					fy[cy] = cos((PI / h) * (y + 0.5) * cy);
				for (let cy = 0, j = 0; cy < ly; cy++)
					for (let cx = cy ? 0 : 1, fy2 = fy[cy] * 2; cx * ly < lx * (ly - cy); cx++, j++)
						l += l_ac[j] * fx[cx] * fy2;
				for (let cy = 0, j = 0; cy < 3; cy++) {
					for (let cx = cy ? 0 : 1, fy2 = fy[cy] * 2; cx < 3 - cy; cx++, j++) {
						const f = fx[cx] * fy2;
						p += p_ac[j] * f;
						q += q_ac[j] * f;
					}
				}
				if (a_ac)
					for (let cy = 0, j = 0; cy < 5; cy++)
						for (let cx = cy ? 0 : 1, fy2 = fy[cy] * 2; cx < 5 - cy; cx++, j++)
							a += a_ac[j] * fx[cx] * fy2;
				const b = l - (2 / 3) * p;
				const r = (3 * l - b + q) / 2;
				const g = r - q;
				rgba[i] = max(0, 255 * min(1, r));
				rgba[i + 1] = max(0, 255 * min(1, g));
				rgba[i + 2] = max(0, 255 * min(1, b));
				rgba[i + 3] = max(0, 255 * min(1, a));
			}
		}
		return { w, h, rgba };
	}

	function rgbaToDataURL(w: number, h: number, rgba: Uint8Array): string {
		const row = w * 4 + 1;
		const idat = 6 + h * (5 + row);
		const bytes = [
			137,
			80,
			78,
			71,
			13,
			10,
			26,
			10,
			0,
			0,
			0,
			13,
			73,
			72,
			68,
			82,
			0,
			0,
			w >> 8,
			w & 255,
			0,
			0,
			h >> 8,
			h & 255,
			8,
			6,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			idat >>> 24,
			(idat >> 16) & 255,
			(idat >> 8) & 255,
			idat & 255,
			73,
			68,
			65,
			84,
			120,
			1
		];
		const table = [
			0, 498536548, 997073096, 651767980, 1994146192, 1802195444, 1303535960, 1342533948, -306674912,
			-267414716, -690576408, -882789492, -1687895376, -2032938284, -1609899400, -1111625188
		];
		let a = 1;
		let b = 0;
		for (let y = 0, i = 0, end = row - 1; y < h; y++, end += row - 1) {
			bytes.push(y + 1 < h ? 0 : 1, row & 255, row >> 8, ~row & 255, (row >> 8) ^ 255, 0);
			for (b = (b + a) % 65521; i < end; i++) {
				const u = rgba[i] & 255;
				bytes.push(u);
				a = (a + u) % 65521;
				b = (b + a) % 65521;
			}
		}
		bytes.push(
			b >> 8,
			b & 255,
			a >> 8,
			a & 255,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			0,
			73,
			69,
			78,
			68,
			174,
			66,
			96,
			130
		);
		for (const [start, end] of [
			[12, 29],
			[37, 41 + idat]
		] as const) {
			let c = ~0;
			for (let i = start; i < end; i++) {
				c ^= bytes[i];
				c = (c >>> 4) ^ table[c & 15];
				c = (c >>> 4) ^ table[c & 15];
			}
			c = ~c;
			bytes[end] = c >>> 24;
			bytes[end + 1] = (c >> 16) & 255;
			bytes[end + 2] = (c >> 8) & 255;
			bytes[end + 3] = c & 255;
		}
		return 'data:image/png;base64,' + btoa(String.fromCharCode(...bytes));
	}

	// End vendored thumbhash decode

	document.querySelectorAll<HTMLElement>('[data-thumbhash]').forEach((el) => {
		const b64 = el.dataset.thumbhash;
		if (!b64) return;
		try {
			const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
			const { w, h, rgba } = thumbHashToRGBA(bytes);
			el.style.backgroundImage = `url(${rgbaToDataURL(w, h, rgba)})`;
		} catch {
			// Silently ignore malformed hashes — the image will load normally
		}
	});
}

/**
 * Generates a self-executing script expression for inline injection.
 * The result is meant to be placed inside a <script> tag in app.html
 * and substituted via transformPageChunk in hooks.server.ts.
 */
export function createThumbhashExpression(): string {
	return `(${initThumbhashes.toString()})();`;
}
