// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

contract HelloWorld {
    string public message;
    uint public number;
    address public walletAddress;
    bool public flag;
    bytes32 public data;

    constructor(
        string memory initMessage, 
        uint initNumber, 
        address initAddress, 
        bool initFlag, 
        bytes32 initData
    ) {
        message = initMessage;
        number = initNumber;
        walletAddress = initAddress;
        flag = initFlag;
        data = initData;
    }

    function getTest() pure public returns(uint) {
        return 42;
    }

    function updateMessage(string memory newMessage) public {
        message = newMessage;
    }

    function updateNumber(uint newNumber) public {
        number = newNumber;
    }

    function updateAddress(address newAddress) public {
        walletAddress = newAddress;
    }

    function updateFlag(bool newFlag) public {
        flag = newFlag;
    }

    function updateData(bytes32 newData) public {
        data = newData;
    }
}
