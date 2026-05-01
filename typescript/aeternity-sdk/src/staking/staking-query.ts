import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import {
  VALIDATOR_STAKING_ACI,
  FRAUD_SLASHER_ACI,
} from '../aci/index.js';

export async function getStakingConfig(
  sdk: AeSdk,
  address: string,
): Promise<{
  owner: string;
  minStake: bigint;
  unstakeDelay: bigint;
  totalStaked: bigint;
  deployedBlock: number;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [VALIDATOR_STAKING_ACI],
    address: address as `ct_${string}`,
  });

  const [ownerRes, minStakeRes, unstakeDelayRes, totalStakedRes, deployedBlockRes] =
    await Promise.all([
      contract.owner(),
      contract.min_stake(),
      contract.unstake_delay(),
      contract.get_total_staked(),
      contract.deployed_block(),
    ]);

  return {
    owner: ownerRes.decodedResult,
    minStake: BigInt(minStakeRes.decodedResult),
    unstakeDelay: BigInt(unstakeDelayRes.decodedResult),
    totalStaked: BigInt(totalStakedRes.decodedResult),
    deployedBlock: Number(deployedBlockRes.decodedResult),
  };
}

export async function getActiveValidators(
  sdk: AeSdk,
  address: string,
): Promise<string[]> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [VALIDATOR_STAKING_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.get_active_validators();
  return result.decodedResult;
}

export async function isActiveValidator(
  sdk: AeSdk,
  address: string,
  validator: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [VALIDATOR_STAKING_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.is_active_validator(validator);
  return Boolean(result.decodedResult);
}

export async function getValidatorWeight(
  sdk: AeSdk,
  address: string,
  validator: string,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [VALIDATOR_STAKING_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.get_validator_weight(validator);
  return BigInt(result.decodedResult);
}

export async function getFraudSlasherConfig(
  sdk: AeSdk,
  address: string,
): Promise<{
  deployedBlock: number;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [FRAUD_SLASHER_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.deployed_block();
  return { deployedBlock: Number(result.decodedResult) };
}
