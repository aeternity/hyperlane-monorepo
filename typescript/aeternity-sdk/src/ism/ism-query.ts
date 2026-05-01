import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import {
  MULTISIG_ISM_ACI,
  AMOUNT_ROUTING_ISM_ACI,
  TIMELOCK_DOMAIN_ROUTING_ISM_ACI,
} from '../aci/index.js';

export async function getIsmType(
  sdk: AeSdk,
  ismAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MULTISIG_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const result = await contract.module_type();
  return Number(result.decodedResult);
}

export async function getMultisigIsmConfig(
  sdk: AeSdk,
  ismAddress: string,
): Promise<{
  validators: string[];
  threshold: number;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MULTISIG_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const [validatorsResult, thresholdResult] = await Promise.all([
    contract.get_validators(),
    contract.get_threshold(),
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
  sender: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MULTISIG_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const result = await contract.verify(metadata, message, sender);
  return result.decodedResult;
}

export async function getAmountRoutingIsmConfig(
  sdk: AeSdk,
  ismAddress: string,
): Promise<{
  owner: string;
  lowerIsm: string;
  upperIsm: string;
  threshold: bigint;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [AMOUNT_ROUTING_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const [ownerRes, lowerRes, upperRes, thresholdRes] = await Promise.all([
    contract.owner(),
    contract.get_lower_ism(),
    contract.get_upper_ism(),
    contract.get_threshold(),
  ]);

  return {
    owner: ownerRes.decodedResult,
    lowerIsm: lowerRes.decodedResult,
    upperIsm: upperRes.decodedResult,
    threshold: BigInt(thresholdRes.decodedResult),
  };
}

export async function getTimelockDomainRoutingIsmConfig(
  sdk: AeSdk,
  ismAddress: string,
): Promise<{
  owner: string;
  timelock: bigint;
  defaultIsm: string | null;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [TIMELOCK_DOMAIN_ROUTING_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const [ownerRes, timelockRes, defaultIsmRes] = await Promise.all([
    contract.owner(),
    contract.get_timelock(),
    contract.get_default_ism(),
  ]);

  return {
    owner: ownerRes.decodedResult,
    timelock: BigInt(timelockRes.decodedResult),
    defaultIsm: defaultIsmRes.decodedResult ?? null,
  };
}

export async function getTimelockPendingChange(
  sdk: AeSdk,
  ismAddress: string,
  domain: number,
): Promise<{ ism: string; executeAfter: bigint } | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [TIMELOCK_DOMAIN_ROUTING_ISM_ACI],
    address: ismAddress as `ct_${string}`,
  });

  const result = await contract.get_pending_change(domain);
  const decoded = result.decodedResult;
  if (!decoded) return null;

  return {
    ism: decoded.ism,
    executeAfter: BigInt(decoded.execute_after),
  };
}
