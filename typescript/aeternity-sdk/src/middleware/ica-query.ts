import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { INTERCHAIN_ACCOUNT_ROUTER_ACI } from '../aci/index.js';

export async function getIcaRouterConfig(
  sdk: AeSdk,
  address: string,
): Promise<{ owner: string; deployedBlock: number }> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [INTERCHAIN_ACCOUNT_ROUTER_ACI],
    address: address as `ct_${string}`,
  });

  const [ownerRes, blockRes] = await Promise.all([
    contract.owner(),
    contract.deployed_block(),
  ]);

  return {
    owner: ownerRes.decodedResult,
    deployedBlock: Number(blockRes.decodedResult),
  };
}

export async function getLocalIca(
  sdk: AeSdk,
  address: string,
  origin: number,
  owner: string,
): Promise<string | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [INTERCHAIN_ACCOUNT_ROUTER_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.get_local_ica(origin, owner);
  return result.decodedResult ?? null;
}
