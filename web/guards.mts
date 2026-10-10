/** Shape checks narrow opaque host input without inventing application schemas. */
export const object = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
export const keys = (value: unknown, allowed: readonly string[]): value is Record<string, unknown> =>
  object(value) && Object.keys(value).every(key => allowed.includes(key));
export const own = (value: unknown, key: PropertyKey): boolean =>
  value !== null && typeof value === 'object' && Object.hasOwn(value, key);
export const integer = (value: unknown): value is number => typeof value === 'number' && Number.isInteger(value);
export const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);
