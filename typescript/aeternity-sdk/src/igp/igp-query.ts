import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { IGP_ACI } from '../aci/index.js';

export async function getIgpConfig(
  sdk: AeSdk,
  igpAddress: string,
): Promise<{
  owner: string;
  beneficiary: string;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [IGP_ACI],
    address: igpAddress as `ct_${string}`,
  });

  const [ownerResult, beneficiaryResult] = await Promise.all([
    contract.get_owner(),
    contract.get_beneficiary(),
  ]);

  return {
    owner: ownerResult.decodedResult,
    beneficiary: beneficiaryResult.decodedResult,
  };
}

export async function quoteGasPayment(
  sdk: AeSdk,
  igpAddress: string,
  destinationDomain: number,
  gasAmount: bigint,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [IGP_ACI],
    address: igpAddress as `ct_${string}`,
  });

  const result = await contract.quote_gas_payment(destinationDomain, gasAmount);
  return BigInt(result.decodedResult);
}

export async function getDestinationGasOverhead(
  sdk: AeSdk,
  igpAddress: string,
  destinationDomain: number,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [IGP_ACI],
    address: igpAddress as `ct_${string}`,
  });

  const result = await contract.destination_gas_overhead(destinationDomain);
  return BigInt(result.decodedResult);
}
