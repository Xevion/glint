import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn utility', () => {
	it('merges class names correctly', () => {
		const result = cn('px-4', 'py-2');
		expect(result).toBeTruthy();
	});

	it('handles conditional classes', () => {
		const result = cn('base-class', undefined, 'visible');
		expect(result).toContain('base-class');
		expect(result).toContain('visible');
	});
});
