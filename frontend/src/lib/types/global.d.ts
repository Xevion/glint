interface FileSystemHandle {
	readonly kind: 'file' | 'directory';
	readonly name: string;
	isSameEntry(other: FileSystemHandle): Promise<boolean>;
}

interface FileSystemFileHandle extends FileSystemHandle {
	readonly kind: 'file';
	getFile(): Promise<File>;
	createWritable(options?: { keepExistingData?: boolean }): Promise<FileSystemWritableFileStream>;
}

interface FileSystemDirectoryHandle extends FileSystemHandle {
	readonly kind: 'directory';
	getFileHandle(name: string, options?: { create?: boolean }): Promise<FileSystemFileHandle>;
	getDirectoryHandle(
		name: string,
		options?: { create?: boolean }
	): Promise<FileSystemDirectoryHandle>;
	removeEntry(name: string, options?: { recursive?: boolean }): Promise<void>;
	resolve(possibleDescendant: FileSystemHandle): Promise<string[] | null>;
	values(): AsyncIterableIterator<FileSystemFileHandle | FileSystemDirectoryHandle>;
	keys(): AsyncIterableIterator<string>;
	entries(): AsyncIterableIterator<[string, FileSystemFileHandle | FileSystemDirectoryHandle]>;
	[Symbol.asyncIterator](): AsyncIterableIterator<
		[string, FileSystemFileHandle | FileSystemDirectoryHandle]
	>;
}

interface FileSystemWritableFileStream extends WritableStream {
	write(data: BufferSource | Blob | string | WriteParams): Promise<void>;
	seek(position: number): Promise<void>;
	truncate(size: number): Promise<void>;
}

interface WriteParams {
	type: 'write' | 'seek' | 'truncate';
	data?: BufferSource | Blob | string;
	position?: number;
	size?: number;
}

declare global {
	interface Window {
		showDirectoryPicker(): Promise<FileSystemDirectoryHandle>;
	}
}

export {};
