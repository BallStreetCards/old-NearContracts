//to run : node filename.js
const express = require("express");
const nearAPI = require("near-api-js");
require("dotenv").config();
const app = express();
const port = 3000;
const { connect, KeyPair, keyStores, WalletConnection } = nearAPI;

const fs = require("fs");
const { async } = require("regenerator-runtime");
const { response } = require("express");
const homedir = require("os").homedir();

const ACCOUNT_ID = `${process.env.CONTRACT_NAME}`; // NEAR account tied to the keyPair
const NETWORK_ID = `${process.env.NODE_ENV}`;
// path to your custom keyPair location (ex. function access key for example account)
const KEY_PATH = `/.near-credentials/testnet/${process.env.CONTRACT_NAME}.json`;

const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const config = {
  networkId: "testnet",
  keyStore,
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://wallet.testnet.near.org",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://explorer.testnet.near.org",
};

async function deploy() {
  // connect to NEAR
  const near = await connect(config);
  const account = await near.account(`${process.env.CONTRACT_NAME}`);

  // gets account balance
  const balance = await account.getAccountBalance();
  console.log("Balance:", balance);

  const response = await account.deployContract(
    fs.readFileSync("../out/main.wasm")
  );
  console.log(response);
}

async function initialize() {
  // connect to NEAR
  const near = await connect(config);
  const account = await near.account(process.env.CONTRACT_NAME);

  // gets account balance
  const balance = await account.getAccountBalance();
  console.log("Balance:", balance);

  const contract = new nearAPI.Contract(
    account, // the account object that is connecting
    process.env.CONTRACT_NAME,
    {
      // name of contract you're connecting to
      viewMethods: ["getMessages"], // view methods do not change state but usually return a value
      changeMethods: ["addMessage"], // change methods modify state
      sender: account, // account object to initialize and sign transactions.
    }
  );

  const response = await contract.initialize(
    {
      owner_id: process.env.CONTRACT_NAME,
      metadata: {
        owner_id: process.env.CONTRACT_NAME,
      },
      total_supply: "1000",
      cost_per_token: "20",
    },
    "300000000000000", // attached GAS (optional)
    "1000000000000000000000000" // attached deposit in yoctoNEAR (optional)
  );
  console.log(response);
}

initialize();
