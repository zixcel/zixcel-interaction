import { test } from 'node:test'
import assert from 'node:assert/strict'
import { validateResource, validateOutcome } from '../dist/contracts.mjs'

test('wire resource guards preserve false/null and reject private data and invalid revisions', () => {
  const value = { contract: {resource_id:'one',contract_revision:'c1',value_schema:{type:'boolean'},
    readable:true,availability:{state:'available'},operations:[]},resource_revision:'r1',value:false }
  assert.deepEqual(validateResource(value), [])
  assert.deepEqual(validateResource({...value, value:null, contract:{...value.contract,value_schema:{type:'null'}}}), [])
  assert.notDeepEqual(validateResource({...value,contract:{...value.contract,readable:false}}), [])
  assert.notDeepEqual(validateResource({...value,resource_revision:''}), [])
  assert.notDeepEqual(validateResource({...value,value:'false'}), [])
  assert.notDeepEqual(validateResource({...value,contract:{...value.contract,operations:[null]}}), [])
})
test('outcome validation never guesses application meaning from HTTP status', () => {
  assert.equal(validateOutcome({status:'Success',value:false}), true)
  for (const status of ['ValidationError','Conflict','Forbidden','Unavailable','NotFound','PreconditionFailed']) {
    assert.equal(validateOutcome({status,reason:'test/reason',issues:[]}), true)
  }
  assert.equal(validateOutcome({status:500}), false)
  assert.equal(validateOutcome({status:'Success'}), false)
  assert.equal(validateOutcome({status:'Forbidden',reason:'x',issues:[],value:'secret'}), false)
})
