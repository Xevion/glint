import type { ShaderDisplayInfo } from './types';

export interface CompareImage {
	url: string;
	shader: ShaderDisplayInfo;
}

/** Mock data for development - will be replaced with API calls */
export function getMockCompareImages(): CompareImage[] {
	return [
		{
			url: '/wallpapers/13.jpg',
			shader: { name: 'BSL', version: '8.2.04', author: 'capttatsu', profile: 'High' }
		},
		{
			url: '/wallpapers/53.jpg',
			shader: { name: 'Complementary', version: '5.2.2', author: 'EminGT', profile: 'Ultra' }
		},
		{
			url: '/wallpapers/31.jpg',
			shader: { name: 'Euphoria Patches', version: '1.2', author: 'SpacEagle17', profile: 'Medium' }
		},
		{
			url: '/wallpapers/73.jpg',
			shader: { name: 'Photon', version: '1.0.3', author: 'sixthsurge', profile: 'Low' }
		},
		{
			url: '/wallpapers/23.jpg',
			shader: { name: 'Kappa', version: '5.2', author: 'RRe36', profile: 'Ultra' }
		},
		{
			url: '/wallpapers/41.jpg',
			shader: { name: 'Nostalgia', version: '5.0', author: 'RRe36', profile: 'High' }
		},
		{
			url: '/wallpapers/61.jpg',
			shader: { name: 'SEUS PTGI', version: 'HRR 3', author: 'Cody', profile: 'Ultra' }
		},
		{
			url: '/wallpapers/19.jpg',
			shader: { name: 'Sildurs Vibrant', version: '1.51', author: 'Sildur', profile: 'Medium' }
		},
		{
			url: '/wallpapers/81.jpg',
			shader: { name: 'MakeUp', version: '8.9', author: 'Flavor', profile: 'High' }
		},
		{
			url: '/wallpapers/11.jpg',
			shader: { name: 'Vanilla Plus', version: '2.1', author: 'Jevicraft', profile: 'Low' }
		},
		{
			url: '/wallpapers/91.jpg',
			shader: { name: 'Rethinking Voxels', version: '0.1 alpha', author: 'gri573', profile: 'Ultra' }
		},
		{
			url: '/wallpapers/37.jpg',
			shader: { name: 'AstraLex', version: '3.1', author: 'LexBoosT', profile: 'High' }
		}
	];
}
