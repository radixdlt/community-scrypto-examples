import {
  RadixDappToolkit,
  DataRequestBuilder,
} from '@radixdlt/radix-dapp-toolkit'


const mynetworkId = 2;

console.log ("network ID", mynetworkId);

// UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES 
 
// change and or update the following definition with the value obtained during publish and initiate actions.
const dAppcomponent = 'component_tdx_2_1cr6q7lkf0kqgj70gdawsfwke7lpjzltqazyqnzqaj3dkngxwkn8urh'
// change and update the folling definition with your own dApp-definitions wallet.
const dAppId = 'account_tdx_2_12x2khwpajm53klweahy84uztccmpk3h8wvrlgeurzwk9gcejzl7mrl'
// change and update the following definition with your own redeemable coin
const delayAddress = 'resource_tdx_2_1t5a8884pvfe23m05gserx6avfsrvafwa6u697m9x9ytzcm66v0uc8n'
// change and update the following definition with the correct radix definition
const xrdAddress = 'resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc'

// UPDATES END 

const amount_input = document.querySelector("#amount_input");
const refreshButtonElement = document.getElementById("refreshwallet");
const performSwapButtonElement = document.getElementById("performswap");

let delayBallance = 0
let clientAddress = "<undefined>"

performSwapButtonElement.textContent = "Refresh wallet first"

amount_input.addEventListener("input", (event) => {
  if (clientAddress == "<undefined>"){
    performSwapButtonElement.textContent = "Refresh wallet first"
  }else{
    performSwapButtonElement.textContent = "Swap "+ event.target.value +" XRD for "+ event.target.value * 5.75 + " DELAY"
  }
});

const radixDappToolkit = RadixDappToolkit({
   dAppDefinitionAddress: dAppId,
   networkId: mynetworkId,
 });

radixDappToolkit.walletApi.setRequestData(
  DataRequestBuilder.persona(),
  DataRequestBuilder.accounts().exactly(1),
);

refreshButtonElement.addEventListener("click", async () => {

  const temp = radixDappToolkit.walletApi.getWalletData();
  if (temp.accounts.length != 0){
    clientAddress = temp.accounts[0].address; 
  } else{

    const result = await radixDappToolkit.walletApi.sendRequest()

    if (result.isErr()) return alert(JSON.stringify(result.error, null, 2));

    clientAddress = result.value.accounts[0].address;
  }

  const getAddressDetails = await radixDappToolkit.gatewayApi.state.getEntityDetailsVaultAggregated(clientAddress);

  
  let fungable_count = getAddressDetails.fungible_resources.total_count;
  var delayVaults
  console.log('Items Count:', fungable_count);

  performSwapButtonElement.textContent = "Swap "+ amount_input.value +" XRD for "+ amount_input.value * 5.75 + " DELAY"

  for (let i = 0; i < fungable_count; i++) {

     if (getAddressDetails.fungible_resources.items[i].resource_address == delayAddress){
 	    delayVaults = getAddressDetails.fungible_resources.items[i].vaults;
		break;
	 };
  }

  delayBallance = 0;
  if (delayVaults != null){
    for (let i = 0; i < delayVaults.total_count; i++) {
	  	let amount = parseFloat(delayVaults.items[i].amount,10);
		  if (!isNaN(amount)){
			  delayBallance += amount
		  }
	  }
  }
 
  document.getElementById('delayamount').innerText = delayBallance    
  
  const getDappDetails = await radixDappToolkit.gatewayApi.state.getEntityDetailsVaultAggregated(dAppcomponent);

  document.getElementById('componentname').innerText = getDappDetails.details.blueprint_name    
  document.getElementById('componentname').innerText = getDappDetails.details.blueprint_name    

  document.getElementById('walletAddress').innerText = clientAddress  

 });

performSwapButtonElement.addEventListener("click", async () => {
		let manifest = `
CALL_METHOD Address("${clientAddress}") "withdraw" Address("${xrdAddress}") Decimal("${amount_input.value}");
TAKE_FROM_WORKTOP Address("${xrdAddress}") Decimal("${amount_input.value}") Bucket("bucket");
CALL_METHOD Address("${dAppcomponent}") "redeem_coin" Bucket("bucket");
CALL_METHOD Address("${clientAddress}") "deposit_batch" Expression("ENTIRE_WORKTOP");
`
//    console.log (manifest)
	
    if (clientAddress == "<undefined>"){
      performSwapButtonElement.textContent = "Refresh wallet first"
    }else{
      const TxDetails = await radixDappToolkit.walletApi.sendTransaction({
        transactionManifest: manifest,
        version: 1,
      });

      if (TxDetails.isErr()) return alert(JSON.stringify(TxDetails.error, null, 2));

//      console.log (TxDetails)
    }
  }
);

