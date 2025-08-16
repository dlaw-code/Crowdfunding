const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

async function deployContract() {
  try {
    // 1. Optimize WASM (Docker)
    console.log('Optimizing WASM...');
    execSync(`docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/optimizer:0.16.0`, { stdio: 'inherit' });

    // 2. Upload WASM (Get Code ID)
    console.log('Uploading contract...');
    const uploadCmd = `xiond tx wasm store ./artifacts/crowdfunded.wasm \
      --chain-id xion-testnet-2 \
      --from $WALLET \
      --node https://rpc.xion-testnet-2.burnt.com:443 \
      --gas-prices 0.001uxion \
      --gas auto \
      --gas-adjustment 1.3 \
      -y --output json`;
    
    const uploadRes = JSON.parse(execSync(uploadCmd, { encoding: 'utf-8' }));
    const txHash = uploadRes.txhash;

    // 3. Get Code ID from transaction
    const codeIdCmd = `xiond query tx ${txHash} \
      --node https://rpc.xion-testnet-2.burnt.com:443 \
      --output json | jq -r '.events[-1].attributes[1].value'`;
    const codeId = execSync(codeIdCmd, { encoding: 'utf-8' }).trim();

    // 4. Instantiate Contract (Get Address)
    const initMsg = { 
      name: "My Crowdfund",
      description: "Test Campaign",
      funding_goal: "1000000",
      deadline: (Date.now() + 7 * 86400 * 1000).toString() // 7 days from now
    };

    const instantiateCmd = `xiond tx wasm instantiate ${codeId} '${JSON.stringify(initMsg)}' \
      --from $WALLET \
      --label "crowdfunded" \
      --no-admin \
      --chain-id xion-testnet-2 \
      --node https://rpc.xion-testnet-2.burnt.com:443 \
      --gas-prices 0.025uxion \
      --gas auto \
      --gas-adjustment 1.3 \
      -y --output json`;
    
    const instantiateRes = JSON.parse(execSync(instantiateCmd, { encoding: 'utf-8' }));
    const contractTxHash = instantiateRes.txhash;

    // 5. Extract Contract Address
    const contractAddrCmd = `xiond query tx ${contractTxHash} \
      --node https://rpc.xion-testnet-2.burnt.com:443 \
      --output json | jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value'`;
    
    const contractAddress = execSync(contractAddrCmd, { encoding: 'utf-8' }).trim();

    // 6. Save to deployed_contracts.json
    const deploymentInfo = {
      network: "xion-testnet-2",
      codeId,
      contractAddress,
      txHash: contractTxHash,
      initMsg,
      timestamp: new Date().toISOString()
    };

    fs.writeFileSync(
      path.join(__dirname, '../deployed_contracts.json'),
      JSON.stringify(deploymentInfo, null, 2)
    );

    console.log(`✅ Contract deployed at: ${contractAddress}`);
    console.log(`📄 Details saved to deployed_contracts.json`);

  } catch (error) {
    console.error('🚨 Deployment failed:', error.message);
    process.exit(1);
  }
}

deployContract();