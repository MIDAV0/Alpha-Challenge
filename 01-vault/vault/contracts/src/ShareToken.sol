// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract ShareToken is ERC20 {
    constructor(address initialOwner)
        ERC20("Share Token", "SHR")
    {}
}
