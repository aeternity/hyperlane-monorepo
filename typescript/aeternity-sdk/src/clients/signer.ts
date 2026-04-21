import { AeSdk, Node, MemoryAccount, Contract } from '@aeternity/aepp-sdk';

import { AltVM } from '@hyperlane-xyz/provider-sdk';
import { assert } from '@hyperlane-xyz/utils';

import {
  MAILBOX_ACI,
  MERKLE_TREE_HOOK_ACI,
  MULTISIG_ISM_ACI,
  VALIDATOR_ANNOUNCE_ACI,
  NOOP_HOOK_ACI,
  AEX9_ACI,
  WARP_ROUTER_ACI,
} from '../aci/index.js';
import { AeternityTransaction, AeternityReceipt } from '../utils/types.js';

import { AeternityProvider } from './provider.js';

const ALL_FUNCTIONS_ACI = [
  MAILBOX_ACI,
  MERKLE_TREE_HOOK_ACI,
  MULTISIG_ISM_ACI,
  VALIDATOR_ANNOUNCE_ACI,
  NOOP_HOOK_ACI,
  AEX9_ACI,
  WARP_ROUTER_ACI,
];

export class AeternitySigner
  extends AeternityProvider
  implements AltVM.ISigner<AeternityTransaction, AeternityReceipt>
{
  private signerAddress: string;

  static async connectWithSigner(
    rpcUrls: string[],
    privateKey: string,
    extraParams?: Record<string, any>,
  ): Promise<AeternitySigner> {
    assert(rpcUrls.length > 0, 'got no rpcUrls');

    const node = new Node(rpcUrls[0]);
    const account = new MemoryAccount(privateKey as `sk_${string}`);
    const sdk = new AeSdk({
      nodes: [{ name: 'node', instance: node }],
      accounts: [account],
    });

    const address = account.address;
    return new AeternitySigner(rpcUrls, node, sdk, address);
  }

  protected constructor(
    rpcUrls: string[],
    node: Node,
    sdk: AeSdk,
    signerAddress: string,
  ) {
    super(rpcUrls, node, sdk);
    this.signerAddress = signerAddress;
  }

  getSignerAddress(): string {
    return this.signerAddress;
  }

  supportsTransactionBatching(): boolean {
    return false;
  }

  async transactionToPrintableJson(
    transaction: AeternityTransaction,
  ): Promise<object> {
    return {
      contractId: transaction.contractId,
      entrypoint: transaction.entrypoint,
      args: transaction.args,
      amount: transaction.options?.amount?.toString() ?? '0',
      gas: transaction.options?.gas ?? 'auto',
    };
  }

  async sendAndConfirmTransaction(
    transaction: AeternityTransaction,
  ): Promise<AeternityReceipt> {
    const matchingAci = ALL_FUNCTIONS_ACI.find((a) =>
      a.contract.functions.some((f: any) => f.name === transaction.entrypoint),
    );
    const contract = await Contract.initialize({
      ...this.sdk.getContext(),
      aci: matchingAci ? [matchingAci] : ALL_FUNCTIONS_ACI,
      address: transaction.contractId as `ct_${string}`,
    });

    const callOptions: any = {};
    if (transaction.options?.amount) {
      callOptions.amount = transaction.options.amount.toString();
    }
    if (transaction.options?.gas) {
      callOptions.gas = transaction.options.gas;
    }

    const result = await contract[transaction.entrypoint](
      ...transaction.args,
      callOptions,
    );

    return {
      hash: result.hash,
      blockHeight: Number((result as any).blockHeight ?? (result.result as any)?.blockHeight ?? 0),
      blockHash: String((result as any).blockHash ?? (result.result as any)?.blockHash ?? ''),
      returnValue: result.decodedResult,
      gasUsed: Number(result.result?.gasUsed ?? 0),
      log: (result.result as any)?.log ?? [],
    };
  }

  async sendAndConfirmBatchTransactions(
    _transactions: AeternityTransaction[],
  ): Promise<AeternityReceipt> {
    throw new Error(
      `${AeternitySigner.name} does not support transaction batching`,
    );
  }
}
