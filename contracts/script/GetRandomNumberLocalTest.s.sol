// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Script} from "forge-std/Script.sol";
import {IAdapter} from "../src/interfaces/IAdapter.sol";
import {GetRandomNumberExample} from "../src/user/examples/GetRandomNumberExample.sol";

contract GetRandomNumberLocalTestScript is Script {
    function run() external {
        GetRandomNumberExample getRandomNumberExample;
        IAdapter adapter;

        uint256 plentyOfEthBalance = vm.envUint("PLENTY_OF_ETH_BALANCE");
        address adapterAddress = vm.envAddress("ADAPTER_ADDRESS");
        uint256 userPrivateKey = vm.envUint("USER_PRIVATE_KEY");

        adapter = IAdapter(adapterAddress);

        vm.startBroadcast(userPrivateKey);
        getRandomNumberExample = new GetRandomNumberExample(
            adapterAddress
        );

        uint64 subId = adapter.createSubscription();

        adapter.fundSubscription{value: plentyOfEthBalance}(subId);

        adapter.addConsumer(subId, address(getRandomNumberExample));

        getRandomNumberExample.getRandomNumber();

        getRandomNumberExample.getRandomNumber();
    }
}
