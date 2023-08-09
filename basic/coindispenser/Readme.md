# CoinDispenser

A coin swap utility to swap any pair.</br> 

Usage examples: </br>
- obtaining fixed price tokens for any kind of game.</br>
- redeeming won tokens for a price.</br>

This project contains a DApp and two webclients.</br>

# requirements
Nodejs/NPM needs to be installed</br>
- <b>note</b>: verify the install version is ok for use with the radix-dapp-toolkit.</br>

For the webclients it is assumed the DApp has been published and initiated.</br>
Publishing can be done at the Radix Dashboard: https://rcnet-v2-dashboard.radixdlt.com/</br>
Run an updated <b>inistantiate.rtm</b> file to run as a raw transaction</br>
Use the obtained component address to update the javascript definitions.

This project is developed for Radix RC2-net using Scrypto 0.11.0.
## Getting started: Webclients

In the `/client/src` folder edit the main.js file:</br>
- update the variables as directed

Enter the `client` folder.</br>
- start the website</br>
        %-> npm install	
        %-> npm run dev

This will start the client webserver @localhost:3000

In the `/admin/src` folder edit the main.js file:</br>
- update the variables as directed

Enter the `admin` folder.</br>
- start the website</br>
        %-> npm install	
        %-> npm run dev

This will start the admin webserver @localhost:3001

## Getting started: dApp
In the `/dapp/coindispenser` folder:</br>
-   Source the sourceme on Linux/Bash for an easy start.</br>

        %-> source sourceme

-   Run the tests and obtain the RTM Radix transaction manifest files.</br>
       
        %-> scrypto test
