import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/**
 * Creates a new object with only the specified keys from the source object.
 * Type-safe runtime equivalent of TypeScript's Pick<T, K>.
 */
export function pick<T extends object, K extends keyof T>(obj: T, keys: readonly K[]): Pick<T, K> {
	const result = {} as Pick<T, K>;
	for (const key of keys) result[key] = obj[key];
	return result;
}

export type WithElementRef<T> = T & {
	ref?: HTMLElement | null;
};

export type WithoutChildrenOrChild<T> = Omit<T, 'children' | 'child'>;

export type WithoutChildren<T> = Omit<T, 'children'>;

export type WithoutChild<T> = Omit<T, 'child'>;
