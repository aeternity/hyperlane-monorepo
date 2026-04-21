import { useCallback } from 'react';

import type {
  TypedTransactionReceipt,
} from '@hyperlane-xyz/sdk/providers/ProviderType';
import type { MultiProviderAdapter } from '@hyperlane-xyz/sdk/providers/MultiProviderAdapter';
import type { ITokenMetadata } from '@hyperlane-xyz/sdk/token/ITokenMetadata';
import type { ChainName } from '@hyperlane-xyz/sdk/types';
import type { WarpTypedTransaction } from '@hyperlane-xyz/sdk/warp/types';

import {
  ChainTransactionFns,
  SwitchNetworkFns,
  WatchAssetFns,
} from './types.js';

export {
  useAeternityAccount,
  useAeternityActiveChain,
  useAeternityConnectFn,
  useAeternityDisconnectFn,
  useAeternityWalletDetails,
  setAeternityWalletState,
} from './aeternityWallet.js';

export type AeternityTxHandler = (params: {
  tx: WarpTypedTransaction;
  chainName: ChainName;
}) => Promise<{ hash: string; confirm: () => Promise<TypedTransactionReceipt> }>;

let registeredTxHandler: AeternityTxHandler | undefined;

export function registerAeternityTxHandler(
  handler: AeternityTxHandler | undefined,
): void {
  registeredTxHandler = handler;
}

export function useAeternitySwitchNetwork(
  _multiProvider: MultiProviderAdapter,
): SwitchNetworkFns {
  const onSwitchNetwork = useCallback(async (chainName: ChainName) => {
    throw new Error(
      `Please manually switch your Aeternity wallet to ${chainName}`,
    );
  }, []);

  return { switchNetwork: onSwitchNetwork };
}

export function useAeternityWatchAsset(
  _multiProvider: MultiProviderAdapter,
): WatchAssetFns {
  const onAddAsset = useCallback(
    async (_token: ITokenMetadata, _activeChainName: ChainName) => {
      throw new Error('Watch asset not available for Aeternity');
    },
    [],
  );

  return { addAsset: onAddAsset };
}

export function useAeternityTransactionFns(
  multiProvider: MultiProviderAdapter,
): ChainTransactionFns {
  const { switchNetwork } = useAeternitySwitchNetwork(multiProvider);

  const onSendTx = useCallback(
    async ({
      tx,
      chainName,
      activeChainName: _,
    }: {
      tx: WarpTypedTransaction;
      chainName: ChainName;
      activeChainName?: ChainName;
    }) => {
      if (!registeredTxHandler) {
        throw new Error(
          'Aeternity wallet context not initialized. Ensure AeternityWalletContext is in the provider tree.',
        );
      }
      return registeredTxHandler({ tx, chainName });
    },
    [],
  );

  const onMultiSendTx = useCallback(async () => {
    throw new Error('Multi Transactions not supported on Aeternity');
  }, []);

  return {
    sendTransaction: onSendTx,
    sendMultiTransaction: onMultiSendTx,
    switchNetwork,
  };
}
