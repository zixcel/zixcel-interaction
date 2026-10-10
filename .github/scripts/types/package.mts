/** Minimal API used from npm's pinned semver implementation. */
export interface SemverApi { valid(version: string): string | null }
