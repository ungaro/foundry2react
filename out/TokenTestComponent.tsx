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

      await contract.write.mint([Variable(Identifier { loc: File(0, 349, 354), name: \, BigInt(356e18)]);
    };

    setup();
  }, []);


const testTransfer = async () => {
  try {
    // VM operation: startPrank(Variable(Identifier { loc: File(0, 432, 437), name: \)
    console.assert(await  MemberAccess(File(0, 459, 473), Variable(Identifier { loc: File(0, 459, 464), name: \"token\" }), Identifier { loc: File(0, 465, 473), name: \"transfer\" })( [Variable(Identifier { loc: File(0, 474, 477), name: \"bob\" }), NumberLiteral(File(0, 479, 485), \"100\", \"18\", None)), 'assertTrue failed');
    const result = await  MemberAccess(File(0, 506, 521), Variable(Identifier { loc: File(0, 506, 511), name: \"token\" }), Identifier { loc: File(0, 512, 521), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 522, 525), name: \"bob\" }));
    console.assert(result === BigInt(528e18), 'assertEq failed:  MemberAccess(File(0, 506, 521), Variable(Identifier { loc: File(0, 506, 511), name: \"token\" }), Identifier { loc: File(0, 512, 521), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 522, 525), name: \"bob\" })) !== BigInt(528e18)');
    const result = await  MemberAccess(File(0, 554, 569), Variable(Identifier { loc: File(0, 554, 559), name: \"token\" }), Identifier { loc: File(0, 560, 569), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 570, 575), name: \"alice\" }));
    console.assert(result === BigInt(578e18), 'assertEq failed:  MemberAccess(File(0, 554, 569), Variable(Identifier { loc: File(0, 554, 559), name: \"token\" }), Identifier { loc: File(0, 560, 569), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 570, 575), name: \"alice\" })) !== BigInt(578e18)');
    // VM operation: stopPrank()
    console.log('testTransfer passed');
  } catch (error) {
    console.error('testTransfer failed:', error);
  }
};

const testFailTransferInsufficientBalance = async () => {
  try {
    // VM operation: prank(Variable(Identifier { loc: File(0, 695, 700), name: \)
    await contract.write.transfer([Variable(Identifier { loc: File(0, 726, 729), name: \, BigInt(731e18)]);
    console.log('testFailTransferInsufficientBalance passed');
  } catch (error) {
    console.error('testFailTransferInsufficientBalance failed:', error);
  }
};

const testApproveAndTransferFrom = async () => {
  try {
    // VM operation: prank(Variable(Identifier { loc: File(0, 816, 821), name: \)
    console.assert(await  MemberAccess(File(0, 843, 856), Variable(Identifier { loc: File(0, 843, 848), name: \"token\" }), Identifier { loc: File(0, 849, 856), name: \"approve\" })( [Variable(Identifier { loc: File(0, 857, 860), name: \"bob\" }), NumberLiteral(File(0, 862, 868), \"100\", \"18\", None)), 'assertTrue failed');
    // VM operation: prank(Variable(Identifier { loc: File(0, 898, 901), name: \)
    console.assert(await  MemberAccess(File(0, 923, 941), Variable(Identifier { loc: File(0, 923, 928), name: \"token\" }), Identifier { loc: File(0, 929, 941), name: \"transferFrom\" })( [Variable(Identifier { loc: File(0, 942, 947), name: \"alice\" }), Variable(Identifier { loc: File(0, 949, 952), name: \"bob\" }), NumberLiteral(File(0, 954, 959), \"50\", \"18\", None)), 'assertTrue failed');
    const result = await  MemberAccess(File(0, 980, 995), Variable(Identifier { loc: File(0, 980, 985), name: \"token\" }), Identifier { loc: File(0, 986, 995), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 996, 999), name: \"bob\" }));
    console.assert(result === BigInt(1002e18), 'assertEq failed:  MemberAccess(File(0, 980, 995), Variable(Identifier { loc: File(0, 980, 985), name: \"token\" }), Identifier { loc: File(0, 986, 995), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 996, 999), name: \"bob\" })) !== BigInt(1002e18)');
    const result = await  MemberAccess(File(0, 1027, 1042), Variable(Identifier { loc: File(0, 1027, 1032), name: \"token\" }), Identifier { loc: File(0, 1033, 1042), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 1043, 1048), name: \"alice\" }));
    console.assert(result === BigInt(1051e18), 'assertEq failed:  MemberAccess(File(0, 1027, 1042), Variable(Identifier { loc: File(0, 1027, 1032), name: \"token\" }), Identifier { loc: File(0, 1033, 1042), name: \"balanceOf\" })( [Variable(Identifier { loc: File(0, 1043, 1048), name: \"alice\" })) !== BigInt(1051e18)');
    const result = await  MemberAccess(File(0, 1077, 1092), Variable(Identifier { loc: File(0, 1077, 1082), name: \"token\" }), Identifier { loc: File(0, 1083, 1092), name: \"allowance\" })( [Variable(Identifier { loc: File(0, 1093, 1098), name: \"alice\" }), Variable(Identifier { loc: File(0, 1100, 1103), name: \"bob\" }));
    console.assert(result === BigInt(1106e18), 'assertEq failed:  MemberAccess(File(0, 1077, 1092), Variable(Identifier { loc: File(0, 1077, 1082), name: \"token\" }), Identifier { loc: File(0, 1083, 1092), name: \"allowance\" })( [Variable(Identifier { loc: File(0, 1093, 1098), name: \"alice\" }), Variable(Identifier { loc: File(0, 1100, 1103), name: \"bob\" })) !== BigInt(1106e18)');
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