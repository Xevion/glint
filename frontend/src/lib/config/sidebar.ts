import {
	Activity,
	Camera,
	HardDrive,
	LayoutDashboard,
	Mountain,
	Palette,
	Settings,
	Users
} from '@lucide/svelte';
import type { Component } from 'svelte';

export interface NavItem {
	label: string;
	href: string;
	icon: Component<{ size?: number; strokeWidth?: number }>;
}

export type SidebarContext = 'home' | 'shaders' | 'scenes' | 'admin';

const homeItems: NavItem[] = [
	{ href: '/shaders', label: 'Shaders', icon: Palette },
	{ href: '/scenes', label: 'Scenes', icon: Mountain }
];

const adminItems: NavItem[] = [
	{ href: '/admin', label: 'Dashboard', icon: LayoutDashboard },
	{ href: '/admin/shaders', label: 'Shaders', icon: Palette },
	{ href: '/admin/scenes', label: 'Scenes', icon: Mountain },
	{ href: '/admin/captures', label: 'Captures', icon: Camera },
	{ href: '/admin/runs', label: 'Runs', icon: Activity },
	{ href: '/admin/storage', label: 'Storage', icon: HardDrive },
	{ href: '/admin/users', label: 'Users', icon: Users },
	{ href: '/admin/settings', label: 'Settings', icon: Settings }
];

export const contextTitle: Record<SidebarContext, string> = {
	home: 'Browse',
	shaders: 'Browse',
	scenes: 'Browse',
	admin: 'Admin'
};

export function getContext(pathname: string): SidebarContext | null {
	if (pathname.startsWith('/admin')) return 'admin';
	if (pathname.startsWith('/compare')) return null;
	if (pathname.startsWith('/shaders')) return 'shaders';
	if (pathname.startsWith('/scenes')) return 'scenes';
	if (pathname === '/') return 'home';
	return null;
}

export function getContextItems(context: SidebarContext): NavItem[] {
	switch (context) {
		case 'home':
		case 'shaders':
		case 'scenes':
			return homeItems;
		case 'admin':
			return adminItems;
	}
}

export function isActive(pathname: string, href: string): boolean {
	if (href === '/') return pathname === '/';
	if (href === '/admin') return pathname === '/admin';
	return pathname.startsWith(href);
}
