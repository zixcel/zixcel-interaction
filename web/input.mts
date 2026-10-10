// Owner-declared input presentation over the existing ActionContract value algebra.
// No source binding, default value, renderer code, storage, authorization or meaning.
import {reference, validateResource, validateValue, validSchema} from './contracts.mjs'
import { object, keys } from './guards.mjs'
import type { InputDeclaration, ValueSchema } from './types/contracts.mjs'
export type { InputField, InputDeclaration } from './types/contracts.mjs'
export const inputLimits=Object.freeze({fields:32,choices:64,stringBytes:4096,submissionBytes:16384,contractBytes:32768,depth:1})
const bytes=(v: string)=>new TextEncoder().encode(v).length
const bounded=(v: unknown,limit: number)=>{try{const encoded=JSON.stringify(v);return typeof encoded==='string'&&bytes(encoded)<=limit}catch{return false}}
const scalar=(s: ValueSchema): s is Extract<ValueSchema, {type: 'string' | 'boolean' | 'number' | 'integer'}>=>['string','boolean','number','integer'].includes(s.type)

export function validateInputDeclaration(d: unknown): string[]{
  const invalid=['input/declaration/invalid']
  if(!bounded(d,inputLimits.contractBytes)||!keys(d,['action','fields'])||!object(d.action)||!object(d.fields))return invalid
  const a=d.action,s=a.input_schema
  if(validateResource({contract:{resource_id:a.target,contract_revision:a.contract_revision,
    value_schema:{type:'null'},readable:false,availability:{state:'available'},operations:[a]},resource_revision:a.contract_revision}).length
    ||!validSchema(s)||s.type!=='object'||Object.keys(s.fields).length>inputLimits.fields)return invalid
  if(Object.keys(d.fields).length!==Object.keys(s.fields).length)return invalid
  for(const [id,f] of Object.entries(d.fields)){
    const v=Object.hasOwn(s.fields,id)?s.fields[id]:null
    if(!reference(id)||!v||!scalar(v)||!keys(f,['label','sensitive','choices'])||!reference(f.label)||typeof f.sensitive!=='boolean')return invalid
    if(v.type==='string'){
      if((v.max_length??4096)>4096||(v.choices?.length??0)>inputLimits.choices)return invalid
      const choices=v.choices??[]
      if(new Set(choices).size!==choices.length||choices.some(c=>bytes(c)>inputLimits.stringBytes||!validateValue(v,c)))return invalid
      if(choices.length){
        if(!Array.isArray(f.choices)||f.choices.length!==choices.length||f.choices.some((c,i)=>
          !keys(c,['value','label'])||c.value!==choices[i]||!reference(c.label)))return invalid
      }else if(Object.hasOwn(f,'choices'))return invalid
    }else if(Object.hasOwn(f,'choices'))return invalid
  }
  return []
}

export function validateInputValues(declaration: unknown,values: unknown): string[]{
  const errors=validateInputDeclaration(declaration)
  if(errors.length)return errors
  if(!bounded(values,inputLimits.submissionBytes)||!object(values))return ['input/limit']
  if(Object.values(values).some(v=>typeof v==='string'&&bytes(v)>inputLimits.stringBytes))return ['input/limit']
  return validateValue((declaration as InputDeclaration).action.input_schema,values)?[]:['input/invalid']
}
