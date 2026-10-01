import type {ActionContract} from './contracts.mjs'
export interface InputField {label:string;sensitive:boolean;choices?:{value:string;label:string}[]}
export interface InputDeclaration {action:ActionContract;fields:Record<string,InputField>}
export const inputLimits:Readonly<{fields:32;choices:64;stringBytes:4096;submissionBytes:16384;contractBytes:32768;depth:1}>
export function validateInputDeclaration(value:unknown):string[]
export function validateInputValues(declaration:unknown,values:unknown):string[]
