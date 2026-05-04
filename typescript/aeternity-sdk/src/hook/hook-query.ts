import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import {
  IGP_ACI,
  MERKLE_TREE_HOOK_ACI,
  NOOP_HOOK_ACI,
  DOMAIN_ROUTING_HOOK_ACI,
  PAUSABLE_HOOK_ACI,
  RATE_LIMITED_HOOK_ACI,
  LINEAR_FEE_ACI,
} from '../aci/index.js';
import { HOOK_TYPE_NUMBERS } from '../utils/types.js';

export async function getHookType(
  sdk: AeSdk,
  hookAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MERKLE_TREE_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });

  const result = await contract.hook_type();
  return Number(result.decodedResult);
}

export async function getMerkleTreeHookConfig(
  sdk: AeSdk,
  hookAddress: string,
): Promise<{
  count: number;
  root: string;
  latestCheckpoint: { root: string; index: number };
  mailbox: string;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MERKLE_TREE_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });

  const [countResult, rootResult, checkpointResult, mailboxResult] =
    await Promise.all([
      contract.count(),
      contract.root(),
      contract.latest_checkpoint(),
      contract.get_mailbox(),
    ]);

  const checkpoint = checkpointResult.decodedResult;

  return {
    count: Number(countResult.decodedResult),
    root: rootResult.decodedResult,
    latestCheckpoint: {
      root: checkpoint[0],
      index: Number(checkpoint[1]),
    },
    mailbox: mailboxResult.decodedResult,
  };
}

export async function getHookQuoteDispatch(
  sdk: AeSdk,
  hookAddress: string,
  metadata?: Uint8Array,
  message?: Uint8Array,
): Promise<bigint> {
  const hookType = await getHookType(sdk, hookAddress);
  let aci;
  switch (hookType) {
    case HOOK_TYPE_NUMBERS.IGP:
      aci = IGP_ACI;
      break;
    case HOOK_TYPE_NUMBERS.RATE_LIMITED:
      aci = RATE_LIMITED_HOOK_ACI;
      break;
    default:
      aci = NOOP_HOOK_ACI;
      break;
  }
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [aci],
    address: hookAddress as `ct_${string}`,
  });

  const result = await contract.quote_dispatch(
    metadata ?? new Uint8Array(0),
    message ?? new Uint8Array(0),
  );
  return BigInt(result.decodedResult);
}

export async function getDomainRoutingHookConfig(
  sdk: AeSdk,
  hookAddress: string,
): Promise<{
  owner: string;
  domains: number[];
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [DOMAIN_ROUTING_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });

  const [ownerResult, domainsResult] = await Promise.all([
    contract.get_owner(),
    contract.get_domains(),
  ]);

  return {
    owner: ownerResult.decodedResult,
    domains: domainsResult.decodedResult.map(Number),
  };
}

export async function isPausableHookPaused(
  sdk: AeSdk,
  hookAddress: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [PAUSABLE_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });

  const result = await contract.is_paused();
  return result.decodedResult;
}

export async function getRateLimitedHookConfig(
  sdk: AeSdk,
  hookAddress: string,
): Promise<{
  owner: string;
  currentLevel: bigint;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [RATE_LIMITED_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });

  const [ownerRes, levelRes] = await Promise.all([
    contract.owner(),
    contract.current_level(),
  ]);

  return {
    owner: ownerRes.decodedResult,
    currentLevel: BigInt(levelRes.decodedResult),
  };
}

export async function getRateLimitedHookLevel(
  sdk: AeSdk,
  hookAddress: string,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [RATE_LIMITED_HOOK_ACI],
    address: hookAddress as `ct_${string}`,
  });
  const result = await contract.current_level();
  return BigInt(result.decodedResult);
}

export async function getLinearFeeConfig(
  sdk: AeSdk,
  feeAddress: string,
): Promise<{ owner: string; maxFee: bigint; halfAmount: bigint }> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [LINEAR_FEE_ACI],
    address: feeAddress as `ct_${string}`,
  });

  const [ownerRes, maxFeeRes, halfAmountRes] = await Promise.all([
    contract.owner(),
    contract.get_max_fee(),
    contract.get_half_amount(),
  ]);

  return {
    owner: ownerRes.decodedResult,
    maxFee: BigInt(maxFeeRes.decodedResult),
    halfAmount: BigInt(halfAmountRes.decodedResult),
  };
}

export async function quoteLinearFee(
  sdk: AeSdk,
  feeAddress: string,
  destination: number,
  recipient: string,
  amount: bigint,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [LINEAR_FEE_ACI],
    address: feeAddress as `ct_${string}`,
  });
  const result = await contract.quote_transfer_fee(
    destination,
    recipient,
    amount.toString(),
  );
  return BigInt(result.decodedResult);
}
