import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { MULTISIG_ISM_ACI } from '../aci/index.js';

export async function getIsmType(
  sdk: AeSdk,
  ismAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: MULTISIG_ISM_ACI,
    address: ismAddress as `ct_${string}`,
  });

  const result = await contract.module_type();
  return Number(result.decodedResult);
}

export async function getMultisigIsmConfig(
  sdk: AeSdk,
  ismAddress: string,
  domain: number,
): Promise<{
  validators: string[];
  threshold: number;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: MULTISIG_ISM_ACI,
    address: ismAddress as `ct_${string}`,
  });

  const [validatorsResult, thresholdResult] = await Promise.all([
    contract.get_validators(domain),
    contract.get_threshold(domain),
  ]);

  return {
    validators: validatorsResult.decodedResult,
    threshold: Number(thresholdResult.decodedResult),
  };
}

export async function verifyIsm(
  sdk: AeSdk,
  ismAddress: string,
  metadata: Uint8Array,
  message: Uint8Array,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: MULTISIG_ISM_ACI,
    address: ismAddress as `ct_${string}`,
  });

  const result = await contract.verify(metadata, message);
  return result.decodedResult;
}
