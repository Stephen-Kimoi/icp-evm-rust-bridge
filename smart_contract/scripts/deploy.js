const { ethers } = require("hardhat");
const hre = require("hardhat");

const main = async () => {
  // Get deployer address and log it
  const [deployer] = await ethers.getSigners();

  console.log("Deploying contract with the account: ", deployer.address);

  // Get contract factory and deploy the contract
  const counter = await hre.ethers.deployContract("Counter"); 
  await counter.waitForDeployment(); 

  console.log("Counter contract address: ", counter.target);

  // Save the contract's artifacts and address
  saveArtifactsAndAddress(counter);
};

const saveArtifactsAndAddress = (counter) => {
  const fs = require("fs");
  const contractsDir = __dirname + "/../artifacts";

  if (!fs.existsSync(contractsDir)) {
    fs.mkdirSync(contractsDir);
  }

  fs.writeFileSync(
    contractsDir + '/contracts-address.json',
    JSON.stringify({ Counter: counter.address }, undefined, 2)
  );

  const CounterArtifact = hre.artifacts.readArtifactSync("Counter");

  fs.writeFileSync(
    contractsDir + '/Counter.json',
    JSON.stringify(CounterArtifact, null, 2)
  );
};

main().then(() => process.exit(0)).catch(error => {
  console.error(error);
  process.exit(1);
});