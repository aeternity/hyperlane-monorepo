import { AeternityTransaction } from '../utils/types.js';

export function buildStakeTx(
  address: string,
  signingKey: string,
  stakeAmount: bigint,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'stake',
    args: [signingKey],
    options: { amount: stakeAmount },
  };
}

export function buildInitiateUnstakeTx(
  address: string,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'initiate_unstake', args: [] };
}

export function buildCompleteUnstakeTx(
  address: string,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'complete_unstake', args: [] };
}

export function buildSetSlasherTx(
  address: string,
  slasher: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'set_slasher',
    args: [slasher],
  };
}

export function buildSetMinStakeTx(
  address: string,
  minStake: bigint,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'set_min_stake',
    args: [minStake.toString()],
  };
}

export function buildSlashForFraudTx(
  slasherAddress: string,
  validator: string,
  proofId: string,
): AeternityTransaction {
  return {
    contractId: slasherAddress,
    entrypoint: 'slash_for_fraud',
    args: [validator, proofId],
  };
}
