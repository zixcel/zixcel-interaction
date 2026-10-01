export type Json = null | boolean | number | string | Json[] | { [key: string]: Json }
export type ValueSchema = { type: 'null' | 'boolean' }
  | { type: 'string'; min_length?: number; max_length?: number; choices?: string[] }
  | { type: 'number' | 'integer'; minimum: number; maximum: number }
  | { type: 'array'; items: ValueSchema; max_items: number }
  | { type: 'object'; fields: Record<string, ValueSchema>; required?: string[] }
export type Availability = { state: 'available' }
  | { state: 'unavailable' | 'forbidden' | 'precondition_failed'; reason: string }
export interface ActionContract {
  operation_id: string; target: string; contract_revision: string; input_schema: ValueSchema
  availability: Availability; expected_revision_required: boolean
}
export interface ResourceContract {
  resource_id: string; contract_revision: string; value_schema: ValueSchema
  readable: boolean; availability: Availability; operations: ActionContract[]
}
export interface ResourceSnapshot { contract: ResourceContract; resource_revision: string; value?: Json }
export interface InvokeRequest {
  operation_id: string; target: string; contract_revision: string; input: Json
  expected_revision: string | null; request_reference: string
}
export type FailureStatus = 'ValidationError' | 'Conflict' | 'Forbidden' | 'Unavailable' | 'NotFound' | 'PreconditionFailed'
export interface Failure { status: FailureStatus; reason: string; issues: { path: string; reason: string }[] }
export type Outcome<T> = { status: 'Success'; value: T } | Failure
export interface Change { resource_id: string; resource_revision: string; contract_revision: string }
export type ChangeBatch = { kind: 'changes'; after: string; cursor: string; changes: Change[] }
  | { kind: 'reset'; cursor: string; reason: string }
export const CONTRACT: 'zixcel://interaction/v1'
export function reference(value: unknown): value is string
export function validSchema(value: unknown): value is ValueSchema
export function validateValue(schema: unknown, value: unknown): boolean
export function validateResource(value: unknown): string[]
export function validateOutcome(value: unknown): value is Outcome<unknown>
