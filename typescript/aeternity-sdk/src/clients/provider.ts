import { AeSdk, Node, Contract } from '@aeternity/aepp-sdk';

import { AltVM } from '@hyperlane-xyz/provider-sdk';
import { assert } from '@hyperlane-xyz/utils';

import { AeternityTransaction } from '../utils/types.js';
import { MAILBOX_ACI, AEX9_ACI, WARP_ROUTER_ACI } from '../aci/index.js';

export class AeternityProvider implements AltVM.IProvider<AeternityTransaction> {
  protected readonly rpcUrls: string[];
  protected readonly node: Node;
  protected sdk: AeSdk;
  protected networkId?: string;

  static async connect(rpcUrls: string[]): Promise<AeternityProvider> {
    assert(rpcUrls.length > 0, 'got no rpcUrls');
    const node = new Node(rpcUrls[0]);
    return new AeternityProvider(rpcUrls, node);
  }

  constructor(rpcUrls: string[], node: Node, sdk?: AeSdk) {
    this.rpcUrls = rpcUrls;
    this.node = node;
    this.sdk = sdk ?? new AeSdk({
      nodes: [{ name: 'node', instance: node }],
    });
  }

  getSdk(): AeSdk {
    return this.sdk;
  }

  getNode(): Node {
    return this.node;
  }

  protected async initContract(aci: any, address: string): Promise<any> {
    const aciArr = Array.isArray(aci) ? aci : [aci];
    return Contract.initialize({
      ...this.sdk.getContext(),
      aci: aciArr,
      address: address as `ct_${string}`,
    });
  }

  async isHealthy(): Promise<boolean> {
    try {
      const status = await this.node.getStatus();
      return !status.syncing;
    } catch {
      return false;
    }
  }

  getRpcUrls(): string[] {
    return this.rpcUrls;
  }

  async getHeight(): Promise<number> {
    const resp = await this.node.getCurrentKeyBlockHeight();
    return typeof resp === 'number' ? resp : Number((resp as any).height ?? resp);
  }

  async getBalance(req: AltVM.ReqGetBalance): Promise<bigint> {
    if (req.denom && !req.denom.startsWith('ak_')) {
      const contract = await this.initContract(AEX9_ACI, req.denom);
      const result = await contract.balance(req.address);
      return BigInt(result.decodedResult ?? 0);
    }
    try {
      const account = await this.node.getAccountByPubkey(req.address);
      return BigInt(account.balance);
    } catch {
      return BigInt(0);
    }
  }

  async getTotalSupply(req: AltVM.ReqGetTotalSupply): Promise<bigint> {
    if (req.denom) {
      const contract = await this.initContract(AEX9_ACI, req.denom);
      const result = await contract.total_supply();
      return BigInt(result.decodedResult);
    }
    throw new Error('Native AE has no total supply');
  }

  async estimateTransactionFee(
    req: AltVM.ReqEstimateTransactionFee<AeternityTransaction>,
  ): Promise<AltVM.ResEstimateTransactionFee> {
    const gasUnits = BigInt(req.transaction.options?.gas ?? 50000);
    const gasPrice = req.transaction.options?.gasPrice ?? 1000000000;
    const fee = gasUnits * BigInt(gasPrice);
    return { gasUnits, gasPrice, fee };
  }

  async isMessageDelivered(req: AltVM.ReqIsMessageDelivered): Promise<boolean> {
    const contract = await this.initContract(MAILBOX_ACI, req.mailboxAddress);
    const result = await contract.delivered(req.messageId);
    return result.decodedResult;
  }

  async getToken(req: AltVM.ReqGetToken): Promise<AltVM.ResGetToken> {
    let ismAddress = '';
    let hookAddress = '';
    let denom = '';
    let tokenType = AltVM.TokenType.native;

    try {
      const routerContract = await this.initContract(WARP_ROUTER_ACI, req.tokenAddress);

      ismAddress = '';
      hookAddress = '';

      try {
        const tokenResult = await routerContract.token?.();
        denom = tokenResult?.decodedResult ?? '';
      } catch {
        denom = '';
      }

      if (!denom || denom === req.tokenAddress) {
        tokenType = denom === req.tokenAddress
          ? AltVM.TokenType.synthetic
          : AltVM.TokenType.native;
      } else {
        tokenType = AltVM.TokenType.collateral;
      }
    } catch {
      tokenType = AltVM.TokenType.native;
    }

    const token: AltVM.ResGetToken = {
      address: req.tokenAddress,
      owner: '',
      tokenType,
      mailboxAddress: '',
      ismAddress,
      hookAddress,
      denom,
      name: '',
      symbol: '',
      decimals: 18,
    };

    if (tokenType !== AltVM.TokenType.native && denom) {
      try {
        const aex9 = await this.initContract(AEX9_ACI, denom);
        const nameResult = await aex9.name();
        const symbolResult = await aex9.symbol();
        const decimalsResult = await aex9.decimals();
        token.name = nameResult.decodedResult;
        token.symbol = symbolResult.decodedResult;
        token.decimals = Number(decimalsResult.decodedResult);
      } catch {
        // Metadata unavailable
      }
    }

    return token;
  }

  async getRemoteRouters(
    req: AltVM.ReqGetRemoteRouters,
  ): Promise<AltVM.ResGetRemoteRouters> {
    const contract = await this.initContract(WARP_ROUTER_ACI, req.tokenAddress);

    const remoteRouters: {
      receiverDomainId: number;
      receiverAddress: string;
      gas: string;
    }[] = [];

    try {
      const domainsResult = await contract.domains?.();
      const domains: number[] = domainsResult?.decodedResult ?? [];

      for (const domain of domains) {
        const routerResult = await contract.get_remote_router(domain);
        const gasResult = await contract.get_destination_gas(domain);

        remoteRouters.push({
          receiverDomainId: Number(domain),
          receiverAddress: routerResult.decodedResult?.toString() ?? '',
          gas: gasResult.decodedResult?.toString() ?? '0',
        });
      }
    } catch {
      // No domains configured
    }

    return {
      address: req.tokenAddress,
      remoteRouters,
    };
  }

  async getBridgedSupply(req: AltVM.ReqGetBridgedSupply): Promise<bigint> {
    const { tokenType, denom } = await this.getToken({
      tokenAddress: req.tokenAddress,
    });

    switch (tokenType) {
      case AltVM.TokenType.native: {
        return this.getBalance({
          address: req.tokenAddress,
          denom: '',
        });
      }
      case AltVM.TokenType.synthetic: {
        return this.getTotalSupply({
          denom: req.tokenAddress,
        });
      }
      case AltVM.TokenType.collateral: {
        return this.getBalance({
          address: req.tokenAddress,
          denom,
        });
      }
      default: {
        throw new Error(`Unknown token type ${tokenType}`);
      }
    }
  }

  async quoteRemoteTransfer(
    req: AltVM.ReqQuoteRemoteTransfer,
  ): Promise<AltVM.ResQuoteRemoteTransfer> {
    assert(req.recipient, 'Aeternity quote remote transfer needs the recipient');
    assert(req.amount, 'Aeternity quote remote transfer needs the amount');

    const contract = await this.initContract(WARP_ROUTER_ACI, req.tokenAddress);

    try {
      const result = await contract.quote_transfer_remote(
        req.destinationDomainId,
        req.recipient,
        BigInt(req.amount!),
      );

      const r = result.decodedResult;
      const total = typeof r === 'object' && r.total_token !== undefined
        ? BigInt(r.total_token)
        : BigInt(r ?? 0);

      return {
        denom: '',
        amount: total,
      };
    } catch {
      return {
        denom: '',
        amount: BigInt(0),
      };
    }
  }
}
