import { AeSdk, Node, MemoryAccount, Contract } from '@aeternity/aepp-sdk';

import { AltVM } from '@hyperlane-xyz/provider-sdk';
import { assert } from '@hyperlane-xyz/utils';

import { AeternityTransaction, AeternityReceipt } from '../utils/types.js';

import { AeternityProvider } from './provider.js';

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
    const account = new MemoryAccount(privateKey as `${string}`);
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
    const contract = await Contract.initialize({
      ...this.sdk.getContext(),
      aci: transaction.args[transaction.args.length]
        ? undefined
        : undefined,
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
      blockHeight: result.blockHeight ?? 0,
      blockHash: result.blockHash ?? '',
      returnValue: result.decodedResult,
      gasUsed: Number(result.result?.gasUsed ?? 0),
      log: result.result?.log ?? [],
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
