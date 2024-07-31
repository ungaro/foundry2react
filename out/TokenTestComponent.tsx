import React, { useState, useEffect } from 'react';
import {
  createPublicClient,
  createWalletClient,
  http,
  parseAbi,
  getContract,
} from "viem";
import { anvil } from "viem/chains";
import { privateKeyToAccount } from "viem/accounts";

const TokenTestTestComponent = () => {
  const [publicClient, setPublicClient] = useState(null);
  const [walletClient, setWalletClient] = useState(null);
  const [contract, setContract] = useState(null);
  const [token, setToken] = useState(null);
  const [alice, setAlice] = useState(null);
  const [bob, setBob] = useState(null);

  useEffect(() => {
    const setup = async () => {
      const account = privateKeyToAccount(import.meta.env.PRIVATE_KEY);
      const tokenAccount = privateKeyToAccount(import.meta.env.TOKEN_PRIVATE_KEY);
      const aliceAccount = privateKeyToAccount(import.meta.env.ALICE_PRIVATE_KEY);
      const bobAccount = privateKeyToAccount(import.meta.env.BOB_PRIVATE_KEY);

      const publicClient = createPublicClient({
        chain: anvil,
        transport: http(import.meta.env.RPC_URL),
      });

      const walletClient = createWalletClient({
        account,
        chain: anvil,
        transport: http(import.meta.env.RPC_URL),
      });

      setPublicClient(publicClient);
      setWalletClient(walletClient);
      setToken(tokenAccount);
      setAlice(aliceAccount);
      setBob(bobAccount);

      const contractABI = parseAbi([
        "function mint(address, address) public",
        "function prank(address) public",
        "function startPrank(address) public",
        "function stopPrank() public",
        "function transfer(address, address) public",
      ]);

      const contract = getContract({
        address: import.meta.env.CONTRACT_ADDRESS,
        abi: contractABI,
        publicClient,
        walletClient,
      });

      setContract(contract);

      await contract.write.mint([alice, BigInt(356)]);
    };

    setup();
  }, []);


const testTransfer = async () => {
  try {
    // VM operation: startPrank(alice)
    console.assert(await unknownFunction11(), 'assertTrue failed');
    const result = await unknownFunction11();
    console.assert(result === BigInt(528), 'assertEq failed: unknownFunction11() !== BigInt(528)');
    const result = await unknownFunction11();
    console.assert(result === BigInt(578), 'assertEq failed: unknownFunction11() !== BigInt(578)');
    // VM operation: stopPrank()
    console.log('testTransfer passed');
  } catch (error) {
    console.error('testTransfer failed:', error);
  }
};

const testFailTransferInsufficientBalance = async () => {
  try {
    // VM operation: prank(alice)
    await contract.write.transfer([bob, BigInt(731)]);
    console.log('testFailTransferInsufficientBalance passed');
  } catch (error) {
    console.error('testFailTransferInsufficientBalance failed:', error);
  }
};

const testApproveAndTransferFrom = async () => {
  try {
    // VM operation: prank(alice)
    console.assert(await unknownFunction11(), 'assertTrue failed');
    // VM operation: prank(bob)
    console.assert(await unknownFunction11(), 'assertTrue failed');
    const result = await unknownFunction11();
    console.assert(result === BigInt(1002), 'assertEq failed: unknownFunction11() !== BigInt(1002)');
    const result = await unknownFunction11();
    console.assert(result === BigInt(1051), 'assertEq failed: unknownFunction11() !== BigInt(1051)');
    const result = await unknownFunction11();
    console.assert(result === BigInt(1106), 'assertEq failed: unknownFunction11() !== BigInt(1106)');
    console.log('testApproveAndTransferFrom passed');
  } catch (error) {
    console.error('testApproveAndTransferFrom failed:', error);
  }
};


  return (
    <div>
      <h1>TokenTest Tests</h1>
  <button onClick={testTransfer}>Run testTransfer</button>
  <button onClick={testFailTransferInsufficientBalance}>Run testFailTransferInsufficientBalance</button>
  <button onClick={testApproveAndTransferFrom}>Run testApproveAndTransferFrom</button>
    </div>
  );
};

export default TokenTestTestComponent;