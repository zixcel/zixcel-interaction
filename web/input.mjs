// Owner-declared input presentation over the existing ActionContract value algebra.
// No source binding, default value, renderer code, storage, authorization or meaning.
import {reference, validateResource, validateValue} from './contracts.mjs'
export const inputLimits=Object.freeze({fields:32,choices:64,stringBytes:4096,submissionBytes:16384,contractBytes:32768,depth:1})
const object=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)
const keys=(v,allowed)=>object(v)&&Object.keys(v).every(k=>allowed.includes(k))
const bytes=v=>new TextEncoder().encode(v).length
const bounded=(v,limit)=>{try{return bytes(JSON.stringify(v))<=limit}catch{return false}}
const scalar=s=>['string','boolean','number','integer'].includes(s?.type)

export function validateInputDeclaration(d){
  const invalid=['input/declaration/invalid']
  if(!bounded(d,inputLimits.contractBytes)||!keys(d,['action','fields'])||!object(d.action)||!object(d.fields))return invalid
  const a=d.action,s=a.input_schema
  if(validateResource({contract:{resource_id:a.target,contract_revision:a.contract_revision,
    value_schema:{type:'null'},readable:false,availability:{state:'available'},operations:[a]},resource_revision:a.contract_revision}).length
    ||s.type!=='object'||Object.keys(s.fields).length>inputLimits.fields)return invalid
  if(Object.keys(d.fields).length!==Object.keys(s.fields).length)return invalid
  for(const [id,f] of Object.entries(d.fields)){
    const v=Object.hasOwn(s.fields,id)?s.fields[id]:null
    if(!reference(id)||!scalar(v)||!keys(f,['label','sensitive','choices'])||!reference(f.label)||typeof f.sensitive!=='boolean')return invalid
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

export function validateInputValues(declaration,values){
  const errors=validateInputDeclaration(declaration)
  if(errors.length)return errors
  if(!bounded(values,inputLimits.submissionBytes)||!object(values))return ['input/limit']
  if(Object.values(values).some(v=>typeof v==='string'&&bytes(v)>inputLimits.stringBytes))return ['input/limit']
  return validateValue(declaration.action.input_schema,values)?[]:['input/invalid']
}
