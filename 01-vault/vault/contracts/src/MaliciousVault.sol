// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./VaultToken.sol";


contract MaliciousVault is ERC4626 {
        
    constructor(VaultToken asset)
        ERC4626(asset) ERC20("Share Token", "SHR")
    {}
}
