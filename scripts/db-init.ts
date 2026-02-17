/**
 * Write local service URLs to backend/.env after `docker compose up`.
 * Only sets DATABASE_URL — R2/MinIO credentials are managed manually.
 */

import { readFile, writeFile } from "fs/promises";

const ENV_FILE = "backend/.env";
const DATABASE_URL = "postgresql://glint:glint@localhost:59490/glint";

let content: string;
try {
	content = await readFile(ENV_FILE, "utf8");
	const regex = /^DATABASE_URL=.*$/m;
	if (regex.test(content)) {
		content = content.replace(regex, `DATABASE_URL=${DATABASE_URL}`);
	} else {
		content = content.trimEnd() + `\nDATABASE_URL=${DATABASE_URL}\n`;
	}
} catch {
	content = `DATABASE_URL=${DATABASE_URL}\n`;
}

await writeFile(ENV_FILE, content);
console.log(`Updated ${ENV_FILE}`);
