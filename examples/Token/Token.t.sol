// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/Token.sol";

contract TokenTest is Test {
    Token public token;
    address public alice = address(0x1);
    address public bob = address(0x2);

    function setUp() public {
        token = new Token("TestToken", "TTK", 18);
        token.mint(alice, 1000e18);
    }

    function testTransfer() public {
        vm.startPrank(alice);
        assertTrue(token.transfer(bob, 100e18));
        assertEq(token.balanceOf(bob), 100e18);
        assertEq(token.balanceOf(alice), 900e18);
        vm.stopPrank();
    }

    function testFailTransferInsufficientBalance() public {
        vm.prank(alice);
        token.transfer(bob, 2000e18);
    }

    function testApproveAndTransferFrom() public {
        vm.prank(alice);
        assertTrue(token.approve(bob, 100e18));
        
        vm.prank(bob);
        assertTrue(token.transferFrom(alice, bob, 50e18));
        assertEq(token.balanceOf(bob), 50e18);
        assertEq(token.balanceOf(alice), 950e18);
        assertEq(token.allowance(alice, bob), 50e18);
    }
}