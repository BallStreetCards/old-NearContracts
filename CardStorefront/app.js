//to run : node filename.js
const express = require("express");
const nearAPI = require("near-api-js");
const app = express();
const port = 3000;

async function main() {
  // creates keyStore using private key in local storage
  // *** REQUIRES SignIn using walletConnection.requestSignIn() ***

  const { connect, keyStores, WalletConnection } = nearAPI;

  const config = {
    networkId: "testnet",
    keyStore: new keyStores.BrowserLocalStorageKeyStore(),
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  };

  // connect to NEAR
  const near = await connect(config);

  // create wallet connection
  const wallet = new WalletConnection(near);

  const signIn = () => {
    wallet.requestSignIn(
      "example-contract.testnet", // contract requesting access
      "Example App", // optional
      "http://YOUR-URL.com/success", // optional
      "http://YOUR-URL.com/failure" // optional
    );
  };

  const signOut = () => {
    wallet.signOut();
  };

  if (wallet.isSignedIn()) {
    doSomething();
  }

  // returns account Id as string
  const walletAccountId = wallet.getAccountId();

  // returns account object for transaction signing
  const walletAccountObj = wallet.account();

  const account = await near.account("akileus0.testnet");

  // gets account balance
  await account.getAccountBalance();
  await account.getAccountDetails();
}

// app.get("/", (req, res) => res.send("Hello World!"));

// app.listen(port, () =>
//   console.log(`Example app listening at http://localhost:${port}`)
// );

//visit localhost:3000
// assuming you have done 1) npm init 2) npm install express
