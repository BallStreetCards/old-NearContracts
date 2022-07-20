//to run : node filename.js
const express = require("express");
const nearAPI = require("near-api-js");
require("dotenv").config();
const app = express();
//cors to fix cors origin, body-parser to fix the post value on the server
const cors = require("cors");
const bodyParser = require("body-parser");

app.use(cors());
app.use(bodyParser.json());

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

app.get("/", (req, res) => {
  res.send("Hello World!");
});

// Tokenized Card Contract deploy function
app.get("/deploy", async (req, res) => {
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
});

// Tokenized Card Initialize function
app.get("/initialize", async (req, res) => {
  // connect to NEAR
  const near = await connect(config);
  const account = await near.account(process.env.CONTRACT_NAME);

  // gets account balance
  const balance = await account.getAccountBalance();
  // console.log("Balance:", balance);

  const contract = new nearAPI.Contract(
    account, // the account object that is connecting
    process.env.CONTRACT_NAME,
    {
      // name of contract you're connecting to
      viewMethods: ["getMessages"], // view methods do not change state but usually return a value
      changeMethods: [
        "new",
        "buy",
        "internal_add_token_to_owner",
        "internal_remove_token_from_owner",
      ], // change methods modify state
      sender: process.env.CONTRACT_NAME, // account object to initialize and sign transactions.
    }
  );

  try {
    const response = await contract.new({
      args: {
        owner_id: process.env.CONTRACT_NAME,
        metadata: {
          spec: "nft-1.0.0",
          name: "tokenized",
          symbol: "TK",
        },
        total_supply: 100,
        cost_per_token: 5,
      },
    });
    console.log(response);
  } catch (error) {
    console.log(error);
  }
});

// deploy();
// initialize();

app.get("/new-wallet/:uid?", async (req, res) => {
  // connect to NEAR
  const near = await connect(config);
  const account = await near.account(`${process.env.CONTRACT_NAME}`);

  console.log(account);
});

app.listen(port, () => {
  console.log(`Example app listening at http://localhost:${port}`);
});
