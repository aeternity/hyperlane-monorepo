import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { TIMELOCK_GOVERNANCE_ACI, MULTISIG_WALLET_ACI } from '../aci/index.js';

export async function getTimelockGovernanceConfig(
  sdk: AeSdk,
  address: string,
): Promise<{
  admin: string;
  minDelay: bigint;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [TIMELOCK_GOVERNANCE_ACI],
    address: address as `ct_${string}`,
  });

  const [adminRes, minDelayRes] = await Promise.all([
    contract.get_admin(),
    contract.get_min_delay(),
  ]);

  return {
    admin: adminRes.decodedResult,
    minDelay: BigInt(minDelayRes.decodedResult),
  };
}

export async function getTimelockOperation(
  sdk: AeSdk,
  address: string,
  opId: string,
): Promise<{
  target: string;
  value: bigint;
  readyAt: bigint;
  status: string;
} | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [TIMELOCK_GOVERNANCE_ACI],
    address: address as `ct_${string}`,
  });

  const result = await contract.get_operation(opId);
  const decoded = result.decodedResult;
  if (!decoded) return null;

  return {
    target: decoded.target,
    value: BigInt(decoded.value),
    readyAt: BigInt(decoded.ready_at),
    status: decoded.status,
  };
}

export async function isTimelockOperationReady(
  sdk: AeSdk,
  address: string,
  opId: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [TIMELOCK_GOVERNANCE_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.is_ready(opId);
  return Boolean(result.decodedResult);
}

export async function getMultiSigConfig(
  sdk: AeSdk,
  address: string,
): Promise<{ owners: string[]; threshold: number }> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MULTISIG_WALLET_ACI],
    address: address as `ct_${string}`,
  });

  const [ownersRes, thresholdRes] = await Promise.all([
    contract.get_owners(),
    contract.get_threshold(),
  ]);

  return {
    owners: ownersRes.decodedResult,
    threshold: Number(thresholdRes.decodedResult),
  };
}

export async function getMultiSigTransaction(
  sdk: AeSdk,
  address: string,
  txId: number,
): Promise<{
  target: string;
  value: bigint;
  executed: boolean;
  confirmCount: number;
} | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MULTISIG_WALLET_ACI],
    address: address as `ct_${string}`,
  });

  const result = await contract.get_transaction(txId);
  const decoded = result.decodedResult;
  if (!decoded) return null;

  return {
    target: decoded.target,
    value: BigInt(decoded.value),
    executed: Boolean(decoded.executed),
    confirmCount: Number(decoded.confirm_count),
  };
}
