import { AeSdk, Node, Contract } from '@aeternity/aepp-sdk';

import { Address } from '@hyperlane-xyz/utils';

import type { MultiProviderAdapter } from '../../providers/MultiProviderAdapter.js';
import type { ChainName } from '../../types.js';
import type { ITokenAdapter, TransferParams } from './ITokenAdapter.js';

const AEX9_ACI = {
  contract: {
    name: 'MintableAEX9',
    kind: 'contract_main',
    payable: false,
    typedefs: [],
    functions: [
      { name: 'name', arguments: [], returns: 'string', stateful: false, payable: false },
      { name: 'symbol', arguments: [], returns: 'string', stateful: false, payable: false },
      { name: 'decimals', arguments: [], returns: 'int', stateful: false, payable: false },
      { name: 'total_supply', arguments: [], returns: 'int', stateful: false, payable: false },
      {
        name: 'balance',
        arguments: [{ name: 'owner', type: 'address' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'transfer',
        arguments: [
          { name: 'to', type: 'address' },
          { name: 'value', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
    ],
  },
};

interface AeternityTransaction {
  contractId: string;
  entrypoint: string;
  args: any[];
  options?: {
    amount?: bigint;
    gas?: number;
    gasPrice?: number;
  };
}

export class AeternityNativeTokenAdapter implements ITokenAdapter<AeternityTransaction> {
  constructor(
    public readonly chainName: ChainName,
    public readonly multiProvider: MultiProviderAdapter,
    public readonly properties: Record<string, any>,
  ) {}

  async getBalance(address: Address): Promise<bigint> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    try {
      const account = await node.getAccountByPubkey(address);
      return BigInt(account.balance);
    } catch {
      return BigInt(0);
    }
  }

  async getTotalSupply(): Promise<bigint | undefined> {
    return undefined;
  }

  async getMetadata(): Promise<{ decimals: number; symbol: string; name: string; totalSupply: string }> {
    return { name: 'Aeternity', symbol: 'AE', decimals: 18, totalSupply: '' };
  }

  async getMinimumTransferAmount(): Promise<bigint> {
    return 0n;
  }

  async isApproveRequired(): Promise<boolean> {
    return false;
  }

  async isRevokeApprovalRequired(): Promise<boolean> {
    return false;
  }

  async populateApproveTx(): Promise<AeternityTransaction> {
    throw new Error('No approval needed for native AE');
  }

  async populateTransferTx(params: TransferParams): Promise<AeternityTransaction> {
    return {
      contractId: '',
      entrypoint: 'spend',
      args: [params.recipient, params.weiAmountOrId],
      options: { amount: BigInt(params.weiAmountOrId) },
    };
  }

  private getRpcUrl(): string {
    const metadata = this.multiProvider.tryGetChainMetadata(this.chainName);
    const rpcUrls = metadata?.rpcUrls;
    if (!rpcUrls || rpcUrls.length === 0) throw new Error(`No RPC URL for ${this.chainName}`);
    return typeof rpcUrls[0] === 'string' ? rpcUrls[0] : rpcUrls[0].http;
  }
}

export class AeternityAEX9TokenAdapter implements ITokenAdapter<AeternityTransaction> {
  private tokenAddress: string;

  constructor(
    public readonly chainName: ChainName,
    public readonly multiProvider: MultiProviderAdapter,
    public readonly properties: { token: string },
  ) {
    this.tokenAddress = properties.token;
  }

  async getBalance(address: Address): Promise<bigint> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    const contract = await Contract.initialize({
      ...sdk.getContext(),
      aci: [AEX9_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
    try {
      const result = await contract.balance(address);
      return BigInt(result.decodedResult ?? 0);
    } catch {
      return BigInt(0);
    }
  }

  async getTotalSupply(): Promise<bigint | undefined> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    const contract = await Contract.initialize({
      ...sdk.getContext(),
      aci: [AEX9_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
    const result = await contract.total_supply();
    return BigInt(result.decodedResult);
  }

  async getMetadata(): Promise<{ decimals: number; symbol: string; name: string; totalSupply: string }> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    const contract = await Contract.initialize({
      ...sdk.getContext(),
      aci: [AEX9_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
    const [nameRes, symbolRes, decimalsRes, supplyRes] = await Promise.all([
      contract.name(),
      contract.symbol(),
      contract.decimals(),
      contract.total_supply(),
    ]);
    return {
      name: nameRes.decodedResult,
      symbol: symbolRes.decodedResult,
      decimals: Number(decimalsRes.decodedResult),
      totalSupply: supplyRes.decodedResult.toString(),
    };
  }

  async getMinimumTransferAmount(): Promise<bigint> {
    return 0n;
  }

  async isApproveRequired(): Promise<boolean> {
    return false;
  }

  async isRevokeApprovalRequired(): Promise<boolean> {
    return false;
  }

  async populateApproveTx(): Promise<AeternityTransaction> {
    throw new Error('No approval needed for AEX-9');
  }

  async populateTransferTx(params: TransferParams): Promise<AeternityTransaction> {
    return {
      contractId: this.tokenAddress,
      entrypoint: 'transfer',
      args: [params.recipient, BigInt(params.weiAmountOrId)],
    };
  }

  private getRpcUrl(): string {
    const metadata = this.multiProvider.tryGetChainMetadata(this.chainName);
    const rpcUrls = metadata?.rpcUrls;
    if (!rpcUrls || rpcUrls.length === 0) throw new Error(`No RPC URL for ${this.chainName}`);
    return typeof rpcUrls[0] === 'string' ? rpcUrls[0] : rpcUrls[0].http;
  }
}
