#!/usr/bin/env zx

/**
 * run following commands for test
 *      npx zx ./test_script.mjs
 *      source .env      
 */

const { setEnvValue, resetEnvs, e } = require('./utils')

resetEnvs()

await $`resim reset`

const tokenXRD = "resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr"
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

output = await e($`resim new-simple-badge`)
const deploy_badge = output[0]

const pub_package = (await e($`resim publish . --owner-badge ${deploy_badge}`))[0]
setEnvValue('package', pub_package);

// Create an empty liquidity pool
output = await e($`resim call-function  ${pub_package} LiquidityPool new 0,${tokenXRD} "LPT" "LP_Token"`)
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

// simulate pool first fee collection
await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} add_collected_fee 30,${tokenXRD}`)

// add same amount fo liquidity from account2
await e($`resim set-default-account ${account2} ${privkey2}`)
await e($`resim call-method ${component} add_liquidity 100,${tokenXRD}`)

// simulate pool second fee collection
await e($`resim set-default-account ${account} ${privkey}`)
await e($`resim call-method ${component} add_collected_fee 50,${tokenXRD}`)

// Remove all liquidity in the pool

await e($`resim set-default-account ${account1} ${privkey1}`)
await e($`resim call-method ${component} remove_liquidity 100,${lp_token}`)

await e($`resim set-default-account ${account2} ${privkey2}`)
await e($`resim call-method ${component} remove_liquidity 76.923076923076923,${lp_token}`)

// Show component and accounts state: we can see more fee earn by account1 as he started eaning fee before account2 joins
await e($`resim show ${component}`, false)
await e($`resim show ${account1}`, false)
await e($`resim show ${account2}`, false)


