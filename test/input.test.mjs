// Owner input declarations: structure only, never source/semantic interpretation.
import test from 'node:test'
import assert from 'node:assert/strict'
import {validateInputDeclaration, validateInputValues} from '../web/input.mjs'
const declaration = () => ({action:{operation_id:'replace',target:'source:one',contract_revision:'exact:one',
  availability:{state:'available'},expected_revision_required:true,
  input_schema:{type:'object',fields:{f1:{type:'string',max_length:80},f2:{type:'string',choices:['c1','c2'],max_length:80}},required:['f1','f2']}},
  fields:{f1:{label:'Value',sensitive:false},f2:{label:'Decision',sensitive:false,choices:[{value:'c1',label:'Same label'},{value:'c2',label:'Same label'}]}}})

test('owner fields preserve exact choices; labels cannot be submitted as references', () => {
  const d=declaration()
  assert.deepEqual(validateInputDeclaration(d),[])
  assert.deepEqual(validateInputValues(d,{f1:'日本語',f2:'c2'}),[])
  assert.ok(validateInputValues(d,{f1:'value',f2:'Same label'}).length)
  assert.ok(validateInputValues(d,{f1:'value',f2:'c2',roleRef:'injected'}).length)
  d.fields.f1.label='Different label'
  assert.deepEqual(validateInputValues(d,{f1:'value',f2:'c2'}),[])
})

test('closed declaration refuses inferred/recursive/code fields and schema mismatch', () => {
  for(const change of [d=>{d.fields.f1.sourcePath='/secret'},d=>{d.fields.f1.component='Remote'},
    d=>{delete d.fields.f1},d=>{d.action.input_schema.fields.f1={type:'object',fields:{}}},
    d=>{d.fields.f2.choices[0].value='not-a-choice'},d=>{d.fields.f1.initial='secret'}]){
    const d=declaration();change(d);assert.ok(validateInputDeclaration(d).length)
  }
})

test('wire byte bounds precede validation; sensitive values are never echoed in errors', () => {
  const d=declaration();d.fields.f1.sensitive=true
  const secret='🔐'.repeat(4097)
  const errors=validateInputValues(d,{f1:secret,f2:'c1'})
  assert.ok(errors.length);assert.ok(!JSON.stringify(errors).includes('🔐'))
  for(let i=0;i<33;i++){d.fields['x'+i]={label:'x',sensitive:false};d.action.input_schema.fields['x'+i]={type:'boolean'}}
  assert.ok(validateInputDeclaration(d).length)
})
