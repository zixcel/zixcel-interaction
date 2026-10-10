import type { SemverApi } from './types/package.mjs'
import assert from 'node:assert/strict'
import {readFileSync,existsSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {createRequire} from 'node:module'
import {resolve} from 'node:path'
const expectedName='@zixcel/interaction', repository='zixcel/zixcel-interaction'
const manifest=record(JSON.parse(readFileSync('package.json','utf8')) as unknown)
assert.equal(manifest.name,expectedName)
assert(!Object.hasOwn(manifest,'private')||manifest.private===false,'private or invalid private flag')
assert.equal(String(record(manifest.repository).url).replace(/^git\+/,'').replace(/\.git$/,''),'https://github.com/'+repository)
assert.equal(execFileSync('npm',['--version'],{encoding:'utf8'}).trim(),'11.12.1')
const require=createRequire(resolve(execFileSync('npm',['root','-g'],{encoding:'utf8'}).trim(),'npm/package.json'))
const semver=require('semver') as SemverApi
assert.equal(typeof manifest.version,'string')
const version=manifest.version as string
assert.equal(semver.valid(version),version)
for(const file of ['.npmrc','.pnpmfile.cjs','.pnpmfile.mjs','pnpm-workspace.yaml','.yarnrc','.yarnrc.yml'])assert(!existsSync(file),'unreviewed project config: '+file)
assert(!manifest.pnpm,'unreviewed manifest pnpm settings')
assert(!Object.hasOwn(manifest,'tag'),'manifest tag override forbidden')
assert(!manifest.publishConfig || Object.keys(record(manifest.publishConfig ?? {})).every(k=>['access','registry'].includes(k)))
assert(!record(manifest.publishConfig ?? {}).registry || /^https:\/\/registry\.npmjs\.org\/?$/.test(String(record(manifest.publishConfig ?? {}).registry)))
assert(!record(manifest.publishConfig ?? {}).access || record(manifest.publishConfig ?? {}).access==='public')
for(const field of ['dependencies','optionalDependencies','peerDependencies','devDependencies']){
 for(const [name,version] of Object.entries(record(manifest[field] ?? {}))){
  assert.equal(typeof version,'string','exact registry version required: '+name)
  assert.equal(semver.valid(version as string),version,'exact registry version required: '+name)
  const response=await fetch('https://registry.npmjs.org/'+encodeURIComponent(name)+'/'+version,{redirect:'error',signal:AbortSignal.timeout(15000)})
  assert.equal(response.status,200,'registry dependency missing: '+name+'@'+version)
  const metadata=record(await response.json() as unknown);assert.equal(metadata.name,name);assert.equal(metadata.version,version)
  assert(/^sha512-[A-Za-z0-9+/]{86}==$/.test(String(record(metadata.dist).integrity)),'registry integrity missing')
 }
}
console.log('PASS: package identity, repository metadata, exact public registry dependencies')

/** Decode external JSON once; object values remain unknown until checked. */
function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === 'object' && !Array.isArray(value))
  return value as Record<string, unknown>
}
