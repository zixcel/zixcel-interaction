// Browser wire guards for the same UI-neutral contracts as the Rust crate.
export const CONTRACT = 'zixcel://interaction/v1'
const own = (v, key) => Object.hasOwn(v, key)
const object = v => v !== null && typeof v === 'object' && !Array.isArray(v)
const keys = (v, allowed) => object(v) && Object.keys(v).every(k => allowed.includes(k))
export const reference = v => typeof v === 'string' && v.length > 0 && new TextEncoder().encode(v).length <= 256 && !/[\u0000-\u001f\u007f-\u009f]/u.test(v)
const statuses = ['ValidationError','Conflict','Forbidden','Unavailable','NotFound','PreconditionFailed']
const availability = v => keys(v, ['state','reason']) && (v.state === 'available'
  ? !own(v, 'reason') : ['unavailable','forbidden','precondition_failed'].includes(v.state) && reference(v.reason))

export function validateOutcome(v) {
  if (!object(v)) return false
  if (v.status === 'Success') return keys(v, ['status','value']) && own(v, 'value')
  return keys(v, ['status','reason','issues']) && statuses.includes(v.status) && reference(v.reason)
    && Array.isArray(v.issues) && v.issues.length <= 64
    && v.issues.every(i => keys(i, ['path','reason']) && typeof i.path === 'string' && reference(i.reason))
}

export function validateResource(v) {
  const errors = []
  if (!keys(v, ['contract','resource_revision','value']) || !reference(v.resource_revision)) return ['resource/invalid']
  const c = v.contract
  if (!keys(c, ['resource_id','contract_revision','value_schema','readable','availability','operations'])
    || !reference(c.resource_id) || !reference(c.contract_revision) || typeof c.readable !== 'boolean'
    || !availability(c.availability) || !Array.isArray(c.operations) || c.operations.length > 128
    || !validSchema(c.value_schema)) return ['resource/contract/invalid']
  const visible = c.readable && c.availability.state === 'available'
  if (visible !== own(v, 'value')) errors.push('resource/visibility/invalid')
  if (visible && !matchesValue(c.value_schema, v.value)) errors.push('resource/value/invalid')
  const ids = new Set()
  for (const a of c.operations) {
    if (!keys(a, ['operation_id','target','contract_revision','input_schema','availability','expected_revision_required'])
      || !reference(a.operation_id) || ids.has(a.operation_id) || a.target !== c.resource_id
      || !reference(a.contract_revision) || !availability(a.availability)
      || typeof a.expected_revision_required !== 'boolean' || !validSchema(a.input_schema)) errors.push('resource/operation/invalid')
    ids.add(a?.operation_id)
  }
  return errors
}

export function validSchema(s, depth = 0, budget = { remaining: 4096 }) {
  if (depth > 16 || --budget.remaining < 0 || !object(s)) return false
  switch (s.type) {
    case 'null': case 'boolean': return keys(s, ['type'])
    case 'string': return keys(s, ['type','min_length','max_length','choices'])
      && Number.isInteger(s.min_length ?? 0) && Number.isInteger(s.max_length ?? 4096)
      && (s.min_length ?? 0) >= 0 && (s.max_length ?? 4096) <= 65536
      && (s.min_length ?? 0) <= (s.max_length ?? 4096)
      && (s.choices === undefined || Array.isArray(s.choices) && s.choices.length <= 256 && s.choices.every(c => typeof c === 'string'))
    case 'number': case 'integer': return keys(s, ['type','minimum','maximum'])
      && Number.isFinite(s.minimum) && Number.isFinite(s.maximum) && s.minimum <= s.maximum
      && (s.type === 'number' || Number.isSafeInteger(s.minimum) && Number.isSafeInteger(s.maximum))
    case 'array': return keys(s, ['type','items','max_items']) && Number.isInteger(s.max_items)
      && s.max_items >= 0 && s.max_items <= 1024 && validSchema(s.items, depth + 1, budget)
    case 'object': return keys(s, ['type','fields','required']) && object(s.fields)
      && Object.keys(s.fields).length <= 128 && Object.values(s.fields).every(f => validSchema(f, depth + 1, budget))
      && (s.required === undefined || Array.isArray(s.required) && s.required.length <= Object.keys(s.fields).length
        && new Set(s.required).size === s.required.length && s.required.every(k => typeof k === 'string' && own(s.fields, k)))
    default: return false
  }
}

export function validateValue(schema, value) {
  return validSchema(schema) && matchesValue(schema, value)
}

function matchesValue(s, v, depth = 0, budget = { remaining: 4096 }) {
  if (depth > 16 || --budget.remaining < 0) return false
  switch (s.type) {
    case 'null': return v === null
    case 'boolean': return typeof v === 'boolean'
    case 'string': return typeof v === 'string' && [...v].length >= (s.min_length ?? 0)
      && [...v].length <= (s.max_length ?? 4096) && (!s.choices?.length || s.choices.includes(v))
    case 'number': case 'integer': return typeof v === 'number' && Number.isFinite(v)
      && v >= s.minimum && v <= s.maximum && (s.type === 'number' || Number.isSafeInteger(v))
    case 'array': return Array.isArray(v) && v.length <= s.max_items && v.every(i => matchesValue(s.items, i, depth + 1, budget))
    case 'object': return object(v) && Object.keys(v).length <= 128 && (s.required ?? []).every(k => own(v, k))
      && Object.entries(v).every(([k, item]) => own(s.fields, k) && matchesValue(s.fields[k], item, depth + 1, budget))
    default: return false
  }
}
