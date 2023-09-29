import {
  RadixDappToolkit,
  DataRequestBuilder,
} from '@radixdlt/radix-dapp-toolkit'


const mynetworkId = 1;

console.log ("network ID", mynetworkId);

// UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES UPDATES 

// change and or update the following definition with the value obtained during publish and initiate actions.
const dAppcomponent = 'component_rdx1crj2jm5l38rht0p9waqer3se3t4932gxxv2e7a40s3pf0h96ea66hp'
// change and update the folling definition with your own dApp-definitions wallet.
const dAppId = 'account_rdx12y7efa9556xfquf6mtn4rq2zmwt0nxsadl29gtfh822h5ag5tlysg6'
// change and update the following definition with your own redeemable coin
const delayAddress = 'resource_rdx1t4cy3r2sdjjnzq8ljdjuv3feypq5mlen4yfvcglswlu30jcqqsswfd'
// change and update the following definition with the correct radix definition
const xrdAddress = 'resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd'
// the swap ratio
const swapratio = 4.475
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
    performSwapButtonElement.textContent = "Swap "+ event.target.value +" DELAY for "+ event.target.value * swapratio + " XRD"
  }
});

document.getElementById('amount_input').max = 9

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

  performSwapButtonElement.textContent = "Swap "+ amount_input.value +" DELAY for "+ amount_input.value * swapratio + " XRD";
  
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
 
  document.getElementById('amount_input').max = delayBallance

  document.getElementById('delayamount').innerText = delayBallance    
  
  const getDappDetails = await radixDappToolkit.gatewayApi.state.getEntityDetailsVaultAggregated(dAppcomponent);

  document.getElementById('componentname').innerText = getDappDetails.details.blueprint_name    
  document.getElementById('componentname').innerText = getDappDetails.details.blueprint_name    

  document.getElementById('walletAddress').innerText = clientAddress  

 });

performSwapButtonElement.addEventListener("click", async () => {
		let manifest = `
CALL_METHOD Address("${clientAddress}") "withdraw" Address("${delayAddress}") Decimal("${amount_input.value}");
TAKE_FROM_WORKTOP Address("${delayAddress}") Decimal("${amount_input.value}") Bucket("bucket");
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

