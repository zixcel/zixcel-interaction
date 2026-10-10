import assert from 'node:assert/strict'
import {validateResource,validateValue,validateOutcome} from '@zixcel/interaction'
import {validateInputDeclaration,validateInputValues} from '@zixcel/interaction/input'
const booleanSchema={type:'boolean'}
const snapshot={contract:{resource_id:'example:settings',contract_revision:'c1',value_schema:booleanSchema,readable:true,availability:{state:'available'},operations:[]},resource_revision:'r1',value:false}
const outcome={status:'Conflict',reason:'example:revision-changed',issues:[]}
assert.deepEqual(validateResource(snapshot),[])
assert.equal(validateValue(booleanSchema,false),true)
assert.equal(validateValue(booleanSchema,'false'),false)
assert.equal(validateOutcome(outcome),true)
assert.equal(outcome.status,'Conflict')
assert.equal(validateOutcome({status:'Success'}),false)
assert.deepEqual(validateResource({...snapshot,value:'false'}),['resource/value/invalid'])
const declaration={action:{operation_id:'example:update',target:'example:settings',contract_revision:'c1',availability:{state:'available'},expected_revision_required:true,input_schema:{type:'object',fields:{enabled:{type:'boolean'}},required:['enabled']}},fields:{enabled:{label:'Enabled',sensitive:false}}}
assert.deepEqual(validateInputDeclaration(declaration),[])
assert.deepEqual(validateInputValues(declaration,{enabled:false}),[])
assert.deepEqual(validateInputValues(declaration,{enabled:'false'}),['input/invalid'])
console.log('PASS: installed root/input API, boolean type rejection and conflict outcome semantics')
