/**
 * Database backup: pg_dump → gzip → R2 upload.
 *
 * Designed for Railway cron: runs once and exits.
 * Reads DATABASE_URL and R2 credentials from environment (or backend/.env locally).
 *
 * Usage:
 *   bun scripts/backup.ts              # Full backup + upload to R2
 *   bun scripts/backup.ts --dry-run    # Dump + compress locally, skip upload
 */

import { existsSync, readFileSync } from "fs";
import { mkdir } from "fs/promises";
import { gzipSync } from "zlib";

/** Load backend/.env when running locally (Railway injects env vars directly) */
function loadEnv(): void {
	const envPath = "backend/.env";
	if (!existsSync(envPath)) return;

	const text = readFileSync(envPath, "utf8");
	for (const line of text.split("\n")) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith("#")) continue;
		const eq = trimmed.indexOf("=");
		if (eq < 0) continue;
		const key = trimmed.slice(0, eq);
		const val = trimmed.slice(eq + 1);
		// Don't override existing env vars (e.g. Railway-injected ones)
		if (!process.env[key]) process.env[key] = val;
	}
}

loadEnv();

function requireEnv(key: string): string {
	const val = process.env[key];
	if (!val) {
		console.error(`missing required env var: ${key}`);
		process.exit(1);
	}
	return val;
}

const DATABASE_URL = requireEnv("DATABASE_URL");

const R2_ACCOUNT_ID = process.env.R2_ACCOUNT_ID ?? "";
const R2_BUCKET = process.env.R2_BUCKET ?? "";
const R2_ACCESS_KEY_ID = process.env.R2_ACCESS_KEY_ID ?? "";
const R2_SECRET_ACCESS_KEY = process.env.R2_SECRET_ACCESS_KEY ?? "";

const R2_ENDPOINT = R2_ACCOUNT_ID
	? `https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com`
	: "";

const BACKUP_PREFIX = "backups/";
const DRY_RUN = process.argv.includes("--dry-run");

function timestamp(): string {
	const d = new Date();
	const pad = (n: number) => String(n).padStart(2, "0");
	return `${d.getUTCFullYear()}${pad(d.getUTCMonth() + 1)}${pad(d.getUTCDate())}T${pad(d.getUTCHours())}${pad(d.getUTCMinutes())}${pad(d.getUTCSeconds())}Z`;
}

function humanBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function elapsedSec(start: number): string {
	return ((Date.now() - start) / 1000).toFixed(1);
}

async function pgDump(): Promise<Buffer> {
	console.log("running pg_dump...");
	const start = Date.now();

	const proc = Bun.spawn(["pg_dump", "--no-owner", "--no-privileges", DATABASE_URL], {
		stdout: "pipe",
		stderr: "pipe",
	});

	const [stdout, stderr] = await Promise.all([
		new Response(proc.stdout).arrayBuffer(),
		new Response(proc.stderr).text(),
	]);
	await proc.exited;

	if (proc.exitCode !== 0) {
		console.error(`pg_dump failed (exit ${proc.exitCode}):\n${stderr}`);
		process.exit(1);
	}

	const raw = Buffer.from(stdout);
	console.log(`pg_dump complete: ${humanBytes(raw.length)} in ${elapsedSec(start)}s`);
	return raw;
}

function compress(data: Buffer): Buffer {
	console.log("compressing...");
	const start = Date.now();
	const compressed = gzipSync(data, { level: 9 });
	const ratio = ((1 - compressed.length / data.length) * 100).toFixed(0);
	console.log(
		`compressed: ${humanBytes(data.length)} -> ${humanBytes(compressed.length)} (${ratio}% reduction) in ${elapsedSec(start)}s`,
	);
	return Buffer.from(compressed);
}

const encoder = new TextEncoder();

async function sha256Hex(data: Uint8Array): Promise<string> {
	const hash = await crypto.subtle.digest("SHA-256", data);
	return [...new Uint8Array(hash)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function hmacSha256(key: ArrayBuffer, data: string): Promise<ArrayBuffer> {
	const k = await crypto.subtle.importKey(
		"raw",
		key,
		{ name: "HMAC", hash: "SHA-256" },
		false,
		["sign"],
	);
	return crypto.subtle.sign("HMAC", k, encoder.encode(data));
}

async function signV4(
	method: string,
	path: string,
	query: string,
	headers: Record<string, string>,
	payloadHash: string,
): Promise<Record<string, string>> {
	const now = new Date();
	const dateStamp = now.toISOString().replace(/[-:]/g, "").slice(0, 8);
	const amzDate = `${dateStamp}T${now.toISOString().replace(/[-:]/g, "").slice(9, 15)}Z`;
	const credentialScope = `${dateStamp}/auto/s3/aws4_request`;

	headers["x-amz-date"] = amzDate;
	headers["x-amz-content-sha256"] = payloadHash;

	const signedHeaderKeys = Object.keys(headers)
		.map((k) => k.toLowerCase())
		.sort();
	const signedHeaders = signedHeaderKeys.join(";");

	const canonicalHeaderLines = signedHeaderKeys
		.map((lk) => {
			const orig = Object.keys(headers).find((h) => h.toLowerCase() === lk)!;
			return `${lk}:${headers[orig].trim()}`;
		})
		.join("\n");

	const canonicalRequest = [method, path, query, `${canonicalHeaderLines}\n`, signedHeaders, payloadHash].join("\n");

	const hashedRequest = await sha256Hex(encoder.encode(canonicalRequest));
	const stringToSign = ["AWS4-HMAC-SHA256", amzDate, credentialScope, hashedRequest].join("\n");

	let signingKey = await hmacSha256(encoder.encode(`AWS4${R2_SECRET_ACCESS_KEY}`).buffer as ArrayBuffer, dateStamp);
	signingKey = await hmacSha256(signingKey, "auto");
	signingKey = await hmacSha256(signingKey, "s3");
	signingKey = await hmacSha256(signingKey, "aws4_request");

	const sig = [...new Uint8Array(await hmacSha256(signingKey, stringToSign))]
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("");

	headers["Authorization"] =
		`AWS4-HMAC-SHA256 Credential=${R2_ACCESS_KEY_ID}/${credentialScope}, SignedHeaders=${signedHeaders}, Signature=${sig}`;

	return headers;
}

async function uploadToR2(key: string, data: Buffer): Promise<void> {
	console.log(`uploading to R2: ${key} (${humanBytes(data.length)})...`);
	const start = Date.now();

	const path = `/${R2_BUCKET}/${key}`;
	const payloadHash = await sha256Hex(data);

	const headers = await signV4("PUT", path, "", {
		Host: `${R2_ACCOUNT_ID}.r2.cloudflarestorage.com`,
		"Content-Type": "application/gzip",
		"Content-Length": String(data.length),
	}, payloadHash);

	const resp = await fetch(`${R2_ENDPOINT}${path}`, {
		method: "PUT",
		headers,
		body: data,
	});

	if (!resp.ok) {
		const body = await resp.text();
		console.error(`R2 upload failed (${resp.status}):\n${body}`);
		process.exit(1);
	}

	console.log(`upload complete in ${elapsedSec(start)}s`);
}

async function verifyUpload(key: string, expectedSize: number): Promise<void> {
	const path = `/${R2_BUCKET}/${key}`;
	const emptyHash = await sha256Hex(new Uint8Array(0));

	const headers = await signV4("HEAD", path, "", {
		Host: `${R2_ACCOUNT_ID}.r2.cloudflarestorage.com`,
	}, emptyHash);

	const resp = await fetch(`${R2_ENDPOINT}${path}`, { method: "HEAD", headers });

	if (!resp.ok) {
		console.error(`verification failed: HEAD returned ${resp.status}`);
		process.exit(1);
	}

	const remoteSize = Number(resp.headers.get("content-length") ?? "0");
	if (remoteSize !== expectedSize) {
		console.error(`verification failed: expected ${expectedSize} bytes, got ${remoteSize}`);
		process.exit(1);
	}

	console.log(`verified: ${key} (${humanBytes(remoteSize)})`);
}

async function main(): Promise<void> {
	const raw = await pgDump();
	const compressed = compress(raw);

	if (DRY_RUN) {
		const dir = "tmp";
		if (!existsSync(dir)) await mkdir(dir, { recursive: true });
		const filename = `glint-${timestamp()}.sql.gz`;
		const outPath = `${dir}/${filename}`;
		await Bun.write(outPath, compressed);
		console.log(`dry-run: saved to ${outPath}`);
		return;
	}

	if (!R2_ENDPOINT) {
		console.error("R2 credentials not configured");
		process.exit(1);
	}

	const key = `${BACKUP_PREFIX}glint-${timestamp()}.sql.gz`;
	await uploadToR2(key, compressed);
	await verifyUpload(key, compressed.length);

	console.log("backup complete");
}

main().catch((err) => {
	console.error("backup failed:", err);
	process.exit(1);
});
