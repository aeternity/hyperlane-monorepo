import { AeSdk, Node, Contract } from '@aeternity/aepp-sdk';

import { Address } from '@hyperlane-xyz/utils';

import type { MultiProviderAdapter } from '../../providers/MultiProviderAdapter.js';
import type { ChainName } from '../../types.js';
import { TokenStandard } from '../TokenStandard.js';

import type {
  IHypTokenAdapter,
  InterchainGasQuote,
  QuoteTransferRemoteParams,
  TransferRemoteParams,
  TransferParams,
} from './ITokenAdapter.js';

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

interface HypTokenAdapterInput {
  standard?: TokenStandard;
  chainName: string;
  addressOrDenom: string;
  collateralAddressOrDenom?: string;
}

const WARP_ROUTER_ACI = {
  contract: {
    name: 'WarpRouter',
    kind: 'contract_main',
    payable: true,
    typedefs: [],
    functions: [
      {
        name: 'transfer_remote',
        arguments: [
          { name: 'destination', type: 'int' },
          { name: 'recipient', type: { bytes: 32 } },
          { name: 'amount', type: 'int' },
        ],
        returns: { bytes: 32 },
        stateful: true,
        payable: true,
      },
      {
        name: 'quote_transfer_remote',
        arguments: [
          { name: 'destination', type: 'int' },
          { name: 'recipient', type: { bytes: 32 } },
          { name: 'amount', type: 'int' },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'get_remote_router',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: { option: [{ bytes: 32 }] },
        stateful: false,
        payable: false,
      },
      {
        name: 'get_destination_gas',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
    ],
  },
};

const AEX9_ACI = {
  contract: {
    name: 'MintableAEX9',
    kind: 'contract_main',
    payable: false,
    typedefs: [],
    functions: [
      {
        name: 'name',
        arguments: [],
        returns: 'string',
        stateful: false,
        payable: false,
      },
      {
        name: 'symbol',
        arguments: [],
        returns: 'string',
        stateful: false,
        payable: false,
      },
      {
        name: 'decimals',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'total_supply',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'balance',
        arguments: [{ name: 'owner', type: 'address' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
    ],
  },
};

export function createAeternityHypAdapter(
  multiProvider: MultiProviderAdapter<{ mailbox?: string }>,
  token: HypTokenAdapterInput,
): IHypTokenAdapter<unknown> | undefined {
  const { standard, chainName, addressOrDenom } = token;
  if (!standard) return undefined;

  switch (standard) {
    case TokenStandard.AeternityHypNative:
      return new AeternityHypNativeAdapter(chainName, multiProvider, {
        token: addressOrDenom,
      });
    case TokenStandard.AeternityHypCollateral:
      return new AeternityHypCollateralAdapter(chainName, multiProvider, {
        token: addressOrDenom,
      });
    case TokenStandard.AeternityHypSynthetic:
      return new AeternityHypSyntheticAdapter(chainName, multiProvider, {
        token: addressOrDenom,
      });
    default:
      return undefined;
  }
}

abstract class BaseAeternityHypAdapter implements IHypTokenAdapter<AeternityTransaction> {
  protected tokenAddress: string;

  constructor(
    public readonly chainName: ChainName,
    public readonly multiProvider: MultiProviderAdapter,
    public readonly properties: { token: string },
  ) {
    this.tokenAddress = properties.token;
  }

  protected getRpcUrl(): string {
    const metadata = this.multiProvider.tryGetChainMetadata(this.chainName);
    const rpcUrls = metadata?.rpcUrls;
    if (!rpcUrls || rpcUrls.length === 0)
      throw new Error(`No RPC URL for ${this.chainName}`);
    return typeof rpcUrls[0] === 'string' ? rpcUrls[0] : rpcUrls[0].http;
  }

  protected async getContract(): Promise<any> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    return Contract.initialize({
      ...sdk.getContext(),
      aci: [WARP_ROUTER_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
  }

  abstract getBalance(address: Address): Promise<bigint>;
  abstract getTotalSupply(): Promise<bigint | undefined>;
  abstract getMetadata(): Promise<{
    decimals: number;
    symbol: string;
    name: string;
    totalSupply: string;
  }>;

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
    throw new Error('No approval needed');
  }

  async populateTransferTx(
    params: TransferParams,
  ): Promise<AeternityTransaction> {
    throw new Error('Use populateTransferRemoteTx for hyp tokens');
  }

  async getDomains(): Promise<number[]> {
    const contract = await this.getContract();
    try {
      const result = await contract.domains?.();
      return result?.decodedResult ?? [];
    } catch {
      return [];
    }
  }

  async getRouterAddress(domain: number): Promise<Buffer> {
    const contract = await this.getContract();
    const result = await contract.get_remote_router(domain);
    const addr = result.decodedResult?.toString() ?? '';
    return Buffer.from(addr, 'hex');
  }

  async getAllRouters(): Promise<Array<{ domain: number; address: Buffer }>> {
    const domains = await this.getDomains();
    const result: Array<{ domain: number; address: Buffer }> = [];
    for (const domain of domains) {
      const router = await this.getRouterAddress(domain);
      result.push({ domain, address: router });
    }
    return result;
  }

  async getBridgedSupply(): Promise<bigint | undefined> {
    return undefined;
  }

  async quoteTransferRemoteGas(
    params: QuoteTransferRemoteParams,
  ): Promise<InterchainGasQuote> {
    const contract = await this.getContract();
    try {
      const result = await contract.quote_transfer_remote(
        params.destination,
        '0x' + '00'.repeat(32),
        0n,
      );
      const amount = BigInt(result.decodedResult ?? 0);
      return { igpQuote: { amount, addressOrDenom: this.tokenAddress } };
    } catch {
      return { igpQuote: { amount: 0n, addressOrDenom: this.tokenAddress } };
    }
  }

  async populateTransferRemoteTx(
    params: TransferRemoteParams,
  ): Promise<AeternityTransaction> {
    const quote = await this.quoteTransferRemoteGas({
      destination: params.destination,
    });
    const totalAmount = BigInt(params.weiAmountOrId) + quote.igpQuote.amount;

    // Left-pad recipient to 32 bytes (Hyperlane bytes32 format)
    const recipientHex = params.recipient.replace(/^0x/i, '').toLowerCase();
    const padded = recipientHex.padStart(64, '0');
    const recipient = '0x' + padded;

    return {
      contractId: this.tokenAddress,
      entrypoint: 'transfer_remote',
      args: [params.destination, recipient, BigInt(params.weiAmountOrId)],
      options: {
        amount: totalAmount,
      },
    };
  }
}

export class AeternityHypNativeAdapter extends BaseAeternityHypAdapter {
  async getBalance(address: Address): Promise<bigint> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    try {
      const account = await node.getAccountByPubkey(address);
      return BigInt(account.balance);
    } catch {
      return 0n;
    }
  }

  async getTotalSupply(): Promise<bigint | undefined> {
    return undefined;
  }

  async getMetadata() {
    return { name: 'Aeternity', symbol: 'AE', decimals: 18, totalSupply: '' };
  }

  async getBridgedSupply(): Promise<bigint | undefined> {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    try {
      const account = await node.getAccountByPubkey(this.tokenAddress);
      return BigInt(account.balance);
    } catch {
      return 0n;
    }
  }
}

export class AeternityHypCollateralAdapter extends BaseAeternityHypAdapter {
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
      return 0n;
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

  async getMetadata() {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    const contract = await Contract.initialize({
      ...sdk.getContext(),
      aci: [AEX9_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
    const [n, s, d, ts] = await Promise.all([
      contract.name(),
      contract.symbol(),
      contract.decimals(),
      contract.total_supply(),
    ]);
    return {
      name: n.decodedResult,
      symbol: s.decodedResult,
      decimals: Number(d.decodedResult),
      totalSupply: ts.decodedResult.toString(),
    };
  }
}

export class AeternityHypSyntheticAdapter extends BaseAeternityHypAdapter {
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
      return 0n;
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

  async getMetadata() {
    const rpcUrl = this.getRpcUrl();
    const node = new Node(rpcUrl);
    const sdk = new AeSdk({ nodes: [{ name: 'node', instance: node }] });
    const contract = await Contract.initialize({
      ...sdk.getContext(),
      aci: [AEX9_ACI],
      address: this.tokenAddress as `ct_${string}`,
    });
    const [n, s, d, ts] = await Promise.all([
      contract.name(),
      contract.symbol(),
      contract.decimals(),
      contract.total_supply(),
    ]);
    return {
      name: n.decodedResult,
      symbol: s.decodedResult,
      decimals: Number(d.decodedResult),
      totalSupply: ts.decodedResult.toString(),
    };
  }

  async getBridgedSupply(): Promise<bigint | undefined> {
    return this.getTotalSupply();
  }
}
