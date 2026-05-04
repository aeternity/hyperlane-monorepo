import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { INTERCHAIN_QUERY_ROUTER_ACI } from '../aci/index.js';

export async function getIcqRouterConfig(
  sdk: AeSdk,
  address: string,
): Promise<{ owner: string; deployedBlock: number }> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [INTERCHAIN_QUERY_ROUTER_ACI],
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
