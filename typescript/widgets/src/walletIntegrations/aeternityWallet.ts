import { useMemo, useSyncExternalStore } from 'react';

import type { MinimalProviderRegistry } from '@hyperlane-xyz/sdk/providers/MinimalProviderRegistry';
import { ProtocolType } from '@hyperlane-xyz/utils';

import type {
  AccountInfo,
  ActiveChainInfo,
  ChainAddress,
  WalletDetails,
} from './types.js';

let aeternityAddress: string | undefined;
let aeternityConnected = false;
let stateVersion = 0;
const listeners = new Set<() => void>();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function getSnapshot(): number {
  return stateVersion;
}

export function setAeternityWalletState(
  address: string | undefined,
  connected: boolean,
): void {
  aeternityAddress = address;
  aeternityConnected = connected;
  stateVersion++;
  listeners.forEach((l) => l());
}

export function useAeternityAccount(
  _multiProvider: MinimalProviderRegistry,
): AccountInfo {
  const version = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return useMemo(() => {
    const addresses: Array<ChainAddress> = [];
    if (aeternityAddress) addresses.push({ address: aeternityAddress });

    return {
      protocol: ProtocolType.Aeternity,
      addresses,
      publicKey: undefined,
      isReady: aeternityConnected && !!aeternityAddress,
    };
  }, [version]);
}

export function useAeternityWalletDetails(): WalletDetails {
  return useMemo(
    () => ({
      name: 'Superhero',
      logoUrl: undefined,
    }),
    [],
  );
}

export function useAeternityConnectFn(): () => void {
  return () => {
    if (typeof window === 'undefined') return;
    // Superhero wallet connection is handled by AeSdkAepp + walletDetector
    // at the application level. This is a placeholder that signals the app
    // should trigger the Superhero wallet connection flow.
    window.dispatchEvent(new CustomEvent('hyperlane:aeternity:connect'));
  };
}

export function useAeternityDisconnectFn(): () => Promise<void> {
  return async () => {
    setAeternityWalletState(undefined, false);
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('hyperlane:aeternity:disconnect'));
    }
  };
}

export function useAeternityActiveChain(
  _multiProvider: MinimalProviderRegistry,
): ActiveChainInfo {
  return useMemo<ActiveChainInfo>(() => ({}), []);
}
