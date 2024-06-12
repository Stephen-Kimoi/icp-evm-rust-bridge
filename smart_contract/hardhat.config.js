require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config(); 

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.18",
  defaultNetwork: "sepolia",
  networks: {
    hardhat: {}, 
    sepolia: {
      url: process.env.ALCHEMY_API_KEY, 
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};