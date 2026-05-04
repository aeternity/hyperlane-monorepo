import {
  type AltVM,
  type ChainMetadataForAltVM,
  type ITransactionSubmitter,
  type MinimumRequiredGasByAction,
  type ProtocolProvider,
  type SignerConfig,
  type TransactionSubmitterConfig,
} from '@hyperlane-xyz/provider-sdk';
import { type IProvider } from '@hyperlane-xyz/provider-sdk/altvm';
import { type IRawHookArtifactManager } from '@hyperlane-xyz/provider-sdk/hook';
import { type IRawIsmArtifactManager } from '@hyperlane-xyz/provider-sdk/ism';
import { type IRawMailboxArtifactManager } from '@hyperlane-xyz/provider-sdk/mailbox';
import {
  type AnnotatedTx,
  type TxReceipt,
} from '@hyperlane-xyz/provider-sdk/module';
import { type IRawWarpArtifactManager } from '@hyperlane-xyz/provider-sdk/warp';
import { type IRawValidatorAnnounceArtifactManager } from '@hyperlane-xyz/provider-sdk/validator-announce';
import { assert } from '@hyperlane-xyz/utils';

import { AeternityProvider } from './provider.js';
import { AeternitySigner } from './signer.js';

export class AeternityProtocolProvider implements ProtocolProvider {
  createProvider(chainMetadata: ChainMetadataForAltVM): Promise<IProvider> {
    assert(chainMetadata.rpcUrls, 'rpc urls undefined');
    const rpcUrls = chainMetadata.rpcUrls.map((rpc) => rpc.http);
    return AeternityProvider.connect(rpcUrls);
  }

  async createSigner(
    chainMetadata: ChainMetadataForAltVM,
    config: SignerConfig,
  ): Promise<AltVM.ISigner<AnnotatedTx, TxReceipt>> {
    assert(chainMetadata.rpcUrls, 'rpc urls undefined');
    const rpcUrls = chainMetadata.rpcUrls.map((rpc) => rpc.http);

    const { privateKey } = config;

    return AeternitySigner.connectWithSigner(rpcUrls, privateKey, {
      metadata: chainMetadata,
    });
  }

  createSubmitter<TConfig extends TransactionSubmitterConfig>(
    _chainMetadata: ChainMetadataForAltVM,
    _config: TConfig,
  ): Promise<ITransactionSubmitter> {
    throw Error('Not implemented');
  }

  createIsmArtifactManager(
    _chainMetadata: ChainMetadataForAltVM,
  ): IRawIsmArtifactManager {
    throw Error('Not implemented');
  }

  createHookArtifactManager(
    _chainMetadata: ChainMetadataForAltVM,
    _context?: { mailbox?: string; proxyAdmin?: string },
  ): IRawHookArtifactManager {
    throw Error('Not implemented');
  }

  createWarpArtifactManager(
    _chainMetadata: ChainMetadataForAltVM,
    _context?: { mailbox?: string },
  ): IRawWarpArtifactManager {
    throw Error('Not implemented');
  }

  createMailboxArtifactManager(
    _chainMetadata: ChainMetadataForAltVM,
  ): IRawMailboxArtifactManager {
    throw Error('Not implemented');
  }

  createValidatorAnnounceArtifactManager(
    _chainMetadata: ChainMetadataForAltVM,
  ): IRawValidatorAnnounceArtifactManager | null {
    throw Error('Not implemented');
  }

  getMinGas(): MinimumRequiredGasByAction {
    return {
      CORE_DEPLOY_GAS: BigInt(1e18),
      WARP_DEPLOY_GAS: BigInt(1e18),
      ISM_DEPLOY_GAS: BigInt(1e18),
      TEST_SEND_GAS: BigInt(1e17),
      AVS_GAS: BigInt(1e17),
    };
  }
}
