# Guess It: A dice roll game 

| **Note** | This example does not use the Public Tets Envirnoment and the official tools for its frontend. Instead, other tools are used to achieve similar functionality.    |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |

### Objective: Guess the closest number from a dice roll
Two players will enter a game and a dice is rolled. The player with the closest guess to the actual dice roll is the winner!

### Prerequisites:
- Having fundamental knowledge of working with shell (all shell commands are prefaced with `$ [COMMAND]`)
- You should be familiar with the [Scrypto](https://docs.radixdlt.com/main/scrypto/introduction.html) coding language and the anatomy of a Radix dApp
- [Install the required toolchain](https://docs.radixdlt.com/main/scrypto/getting-started/install-scrypto.html) which will get Rust and the Radix Engine Simulator going
- Download and install an IDE of your choice with the recommended extensions:
  - VSCode: Install [Rust](https://github.com/rust-lang/vscode-rust) or the [Rust Analyzer](https://github.com/rust-lang/rust-analyzer) extension (known conflicts running both together; using the latter is recommended more highly)
  - IntelliJ Idea: Install [Rust](https://plugins.jetbrains.com/plugin/8182-rust) plugin
- Install NodeJS which comes with the Node Package Manager (NPM)
  - This is only for the front end app 
  - It is recommended to install [NVM](https://github.com/nvm-sh/nvm) first and pull Node in through it
  - [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/#mac-stable) is the recommended package manager as it is async and builds a better dependency tree than NPM
- Install [Insomnia](https://insomnia.rest/download) and import the `insomnia-gameplay.json` collection file for use with testing
- Install [Revup](https://github.com/RadGuild/revup) from RadGuild to run the most basic gameplay scenario 

### Backend Setup:
- Make sure we can build and run tests successfully `$ scrypto test`
- Use `$ revup -r gameplay.rev` and make sure the final line is a success and the game's state is `"state": "Destroyed"`

### Frontend Setup:
- enter the `$ cd server/` folder and install the dependencies using `$ yarn`
- spin up the server using `$ yarn start:dev` or just `$ yarn start`
  - Troubleshoot `port in use` error: run either `$ yarn kill:unix` or `$ yarn kill:windows` to kill the running process
  - Note: Windows kill command is untested

### Gameplay:
- If you've done the frontend setup, open your browser to [http://localhost:3000](http://localhost:3000)
  - You will need 2 browser windows/tabs open to simulate 2 players
  - One player will create a game by clicking the "Create A Game" button
    - A prompt will appear asking you for the game name and a XRD amount you're expecting players to bet
  - Both players should be able to see this newly created game and can click on it to join
  - Once you join, the "Active Game Area" will update with the game state from the contract
  - Once both players have joined, the state will move to "MakeGuess" and a new button will appear
  - Press the MakeGuess button and enter your guess in the following prompt
  - Once both players have guessed, the dice will roll and show up in the "last roll" section of the Active Game Area and a winner will be determined
  - The player who won can then "Claim Funds" by clicking the new button that appears
    - The loser will still see the button (WIP) but cannot withdraw funds
    - If a tie happens, guesses will be reset to 0 and the last_roll will indicate that a roll has taken place
- If you chose to load the insomnia collection
  - Run `Create Player 1`
  - Run `Create Player 2`
  - Run `Create Game`
  - Run `Join Game Player 1`
  - Run `Join Game Player 2`
  - Run `Make Guess Player 1`
  - Run `Make Guess Player 2`
  - Run `Withdraw - Player 2 success`
  - The others can be run at your behest
    - Run `Check state` to view the current contract state
    - Run `Withdraw - Player 1 fails` to see the failure response

### To Do:
- There is no refund mechanism in case players choose to exit the game
- Only one round can be played per contract and the state moves to "Destroyed" and no other actions are available
- Scrypto 0.4 is set to release and have better frontend integration

### Resources
- [Rust Lang](https://doc.rust-lang.org/book)
- [Introduction to Scrypto](https://docs.radixdlt.com/main/scrypto/introduction.html)
- [Scrypto Examples](https://github.com/radixdlt/community-scrypto-examples)

# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.