#!/usr/bin/env zx

/**
 * run following commands for test
 *      npm i -g zx
 *      zx ./script.mjs
 *      source .env      
 */

const { setEnvValue, resetEnvs } = require('./script_utils')

// const regexNewAccount = /(Account component address|Private key|Public key): ([\d|a-z]+)/gm
// const regexNewPackage = /(New Package): ([\d|a-z]+)/gm
// const regexTransaction = /(└─ Component|├─ Component|└─ Resource|├─ Resource): ([\d|a-z]+)/gm
const regexResim = /(New Package|Account component address|Private key|Public key|└─ Component|├─ Component|└─ Resource|├─ Resource): ([\d|a-z]+)/gm

async function e(command, isQuiet = true) {
  let matches = (isQuiet ? await quiet(command) : await command).stdout.matchAll(regexResim)
  let outputs = []
  for (const match of matches) {
    outputs.push(match[2])
  }
  return outputs
}

resetEnvs()
await $`resim reset`

const tokenXRD = "030000000000000000000000000000000000000000000000000004"
setEnvValue('tokenXRD', tokenXRD);

//Setting up 3 account, 1 account for liquidity funding and fee collection simulation, two acccounts for liquidity providers

let output = await e($`resim new-account`)
const account = output[0]
const pubkey = output[1]
const privkey = output[2]
setEnvValue('account', account);
setEnvValue('privkey', privkey);

output = await e($`resim new-account`)
const account1 = output[0]
const pubkey1 = output[1]
const privkey1 = output[2]
setEnvValue('account1', account1);
setEnvValue('privkey1', privkey1);

output = await e($`resim new-account`)
const account2 = output[0]
const pubkey2 = output[1]
const privkey2 = output[2]
setEnvValue('account2', account2);
setEnvValue('privkey2', privkey2);

const pub_package = (await e($`resim publish .`))[0]
setEnvValue('package', pub_package);

output = await e($`resim call-function  ${pub_package} LiquidityPool new 1000,${tokenXRD} "LPT" "LP_Token"`)
const component = output[0]
const lp_mint_bage = output[1]
const lp_token = output[2]
const transient_mint_badge = output[3]
const transient_token = output[4]
setEnvValue('component', component);
setEnvValue('lp_token', lp_token);

// Add liquidity from account1
await e($`resim set-default-account ${account1} ${privkey1}`)
await e($`resim call-method ${component} add_liquidity 100,${tokenXRD}`)

// simulate pool fee collection
await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} add_collected_fee 30,${tokenXRD}`)

// add same amount fo liquidity from account2
await e($`resim set-default-account ${account2} ${privkey2}`)
await e($`resim call-method ${component} add_liquidity 100,${tokenXRD}`)

// simulate pool fee collection
await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} add_collected_fee 50,${tokenXRD}`)

// Removee all liquidity in the pool

await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} remove_liquidity 100,${lp_token}`)

await e($`resim set-default-account ${account1} ${privkey1}`)
await e($`resim call-method ${component} remove_liquidity 100,${lp_token}`)

await e($`resim set-default-account ${account2} ${privkey2}`)
await e($`resim call-method ${component} remove_liquidity 86.9565217391304347,${lp_token}`)

// Add liquidity to th empty pool

await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} add_liquidity 100,${tokenXRD}`)

// Show component and accounts state: we see more fee earn by account1 as he started eaning fee before account2 joins
await e($`resim show ${component}`, false)
await e($`resim show ${account1}`, false)
await e($`resim show ${account2}`, false)
await e($`resim show ${account}`, false)


