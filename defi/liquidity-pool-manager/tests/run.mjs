#!/usr/bin/env zx

/**
 * run following commands for test
 *      npx zx ./run.mjs && source .env      
 */

const { randomInt } = require('crypto');
const { setEnvValue, resetEnvs, e } = require('./utils')

// resetEnvs()

await $`resim reset`

const tokenXRD = "resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr"
setEnvValue('tokenXRD', tokenXRD);
setEnvValue('loanAmount', "100");
setEnvValue('feeAmount', "1");
setEnvValue('repayAmount', "101");

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

const pub_package = (await e($`resim publish .. --owner-badge ${deploy_badge}`))[0]
setEnvValue('package', pub_package);

// Create an empty liquidity pool
output = await e($`resim call-function ${pub_package} FlashLoanService new ${pub_package} ${pub_package} ${tokenXRD} "LPT" "LP_Token"`, false)

const flashloanpool_component = output[0]
const poolmanager_component = output[1]
const service_component = output[2]

//
const transient_mint_badge = output[3]
const transient_token = output[4]
const flashloanpool_admin_badge = output[5]
const lp_mint_badge = output[6]
const poolmanager_admin_badge = output[7]
const lp_token = output[8]

setEnvValue('flashloanpool_component', flashloanpool_component);
setEnvValue('poolmanager_component', poolmanager_component);
setEnvValue('service_component', service_component);
setEnvValue('lp_token', lp_token);
setEnvValue('transient_token', transient_token);

// Add liquidity from account1
await e($`resim set-default-account ${account1} ${privkey1}`)
await e($`resim call-method ${service_component} add_liquidity 900,${tokenXRD}`)

// simulate pool first fee collection
await take_flash_loan(e, 2);

// add same amount fo liquidity from account2
await e($`resim set-default-account ${account2} ${privkey2}`)
await e($`resim call-method ${service_component} add_liquidity 900,${tokenXRD}`)

// simulate pool first fee collection
await take_flash_loan(e, 2);

// Remove all liquidity in the pool
await e($`resim set-default-account ${account1} ${privkey1}`)
await e($`resim call-method ${service_component} remove_liquidity 900,${lp_token}`)



async function take_flash_loan(e, count = 30) {
    await e($`resim set-default-account ${account} ${privkey}`);

    for (let index = 0; index < count; index++) {

        let p = new Promise((resolve) => {

            randomInt(3, async (i) => {

                switch (i) {
                    case 0:
                        resolve(e($`resim run ./rtm/flash_loan_1.rtm`), false);
                        break;

                    case 1:
                        resolve(e($`resim run ./rtm/flash_loan_2.rtm`), false);
                        break;

                    default:
                        resolve(e($`resim run ./rtm/flash_loan_3.rtm`), false);
                        break;
                }

            })

        })

        await p

    }

}