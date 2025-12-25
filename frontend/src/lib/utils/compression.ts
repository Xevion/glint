import { zip } from 'fflate';

const MAX_SIZE = 512 * 1024 * 1024; // 512 MiB

export interface CompressionProgress {
	progress: number; // 0-1
	totalFiles: number;
	processedFiles: number;
	currentFile: string;
}

export interface CompressionResult {
	blob: Blob;
	fileCount: number;
	uncompressedSize: number;
	compressedSize: number;
}

export function validateFileSize(size: number): boolean {
	return size <= MAX_SIZE;
}

export async function calculateDirectorySize(
	dirHandle: FileSystemDirectoryHandle
): Promise<number> {
	let totalSize = 0;

	async function traverseDirectory(handle: FileSystemDirectoryHandle): Promise<void> {
		// @ts-expect-error - FileSystem API types may not be fully available in all environments
		for await (const entry of handle.values()) {
			if (entry.kind === 'file') {
				const file = await (entry as FileSystemFileHandle).getFile();
				totalSize += file.size;
			} else if (entry.kind === 'directory') {
				await traverseDirectory(entry as FileSystemDirectoryHandle);
			}
		}
	}

	await traverseDirectory(dirHandle);
	return totalSize;
}

export async function compressDirectory(
	dirHandle: FileSystemDirectoryHandle,
	onProgress?: (progress: CompressionProgress) => void
): Promise<CompressionResult> {
	const files: Record<string, Uint8Array> = {};
	let totalFiles = 0;
	let processedFiles = 0;
	let currentFile = '';
	let totalUncompressedSize = 0;

	// First pass: count files
	async function countFiles(handle: FileSystemDirectoryHandle): Promise<void> {
		// @ts-expect-error - FileSystem API types may not be fully available in all environments
		for await (const entry of handle.values()) {
			if (entry.kind === 'file') {
				totalFiles++;
			} else if (entry.kind === 'directory') {
				await countFiles(entry as FileSystemDirectoryHandle);
			}
		}
	}

	await countFiles(dirHandle);

	// Second pass: collect file contents
	async function collectFiles(handle: FileSystemDirectoryHandle, path = ''): Promise<void> {
		// @ts-expect-error - FileSystem API types may not be fully available in all environments
		for await (const entry of handle.values()) {
			if (entry.kind === 'file') {
				const file = await (entry as FileSystemFileHandle).getFile();
				const arrayBuffer = await file.arrayBuffer();
				const uint8Array = new Uint8Array(arrayBuffer);
				const filePath = path ? `${path}/${entry.name}` : entry.name;

				files[filePath] = uint8Array;
				totalUncompressedSize += uint8Array.length;

				processedFiles++;
				currentFile = filePath;

				onProgress?.({
					progress: processedFiles / totalFiles,
					totalFiles,
					processedFiles,
					currentFile
				});
			} else if (entry.kind === 'directory') {
				const newPath = path ? `${path}/${entry.name}` : entry.name;
				await collectFiles(entry as FileSystemDirectoryHandle, newPath);
			}
		}
	}

	await collectFiles(dirHandle);

	// Compress using fflate
	const compressed = await new Promise<Uint8Array>((resolve, reject) => {
		zip(files, { level: 6 }, (err, data) => {
			if (err) reject(err);
			else resolve(data);
		});
	});

	const blob = new Blob([compressed as BlobPart], { type: 'application/zip' });

	return {
		blob,
		fileCount: totalFiles,
		uncompressedSize: totalUncompressedSize,
		compressedSize: blob.size
	};
}

export function validateZipFile(file: File): boolean {
	return file.type === 'application/zip' || file.name.endsWith('.zip');
}
