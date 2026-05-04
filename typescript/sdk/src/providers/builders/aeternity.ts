import { Node } from '@aeternity/aepp-sdk';
import { AeternityProvider as AeternitySDKProvider } from '@hyperlane-xyz/aeternity-sdk/runtime';
import { assert } from '@hyperlane-xyz/utils';

import type { RpcUrl } from '../../metadata/chainMetadataTypes.js';
import type { AeternityTypedProvider } from '../ProviderType.js';
import { ProviderType } from '../ProviderType.js';

import type { ProviderBuilderFn } from './types.js';

export const defaultAeternityProviderBuilder: ProviderBuilderFn<
  AeternityTypedProvider
> = (rpcUrls: RpcUrl[], _network: string | number) => {
  const urls = rpcUrls.map((rpc) => rpc.http);
  assert(urls.length > 0, 'Aeternity requires at least one rpcUrl');
  const node = new Node(urls[0]);
  const provider = new AeternitySDKProvider(urls, node);
  return { provider, type: ProviderType.Aeternity };
};
