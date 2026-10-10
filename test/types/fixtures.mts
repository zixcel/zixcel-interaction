import type { ActionContract, InputField, ValueSchema } from '../../web/types/contracts.mjs';
/** Negative fixtures allow only the explicitly tested injected fields. */
export interface FixtureField extends InputField {
  sourcePath?: string; component?: string; initial?: string;
}
export interface DeclarationFixture {
  action: ActionContract & { input_schema: Extract<ValueSchema, { type: 'object' }> };
  fields: Record<string, FixtureField>;
}
export type FixtureMutation = (declaration: DeclarationFixture) => void;
