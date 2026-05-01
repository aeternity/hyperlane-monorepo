import type { AeternityTransaction } from '../utils/types.js';

export function buildPauseWarpRouteTx(routerAddress: string): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'pause', args: [] };
}

export function buildUnpauseWarpRouteTx(routerAddress: string): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'unpause', args: [] };
}

export function buildSetFeeRecipientTx(
  routerAddress: string,
  feeContractAddress: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'set_fee_recipient', args: [feeContractAddress] };
}

export function buildClearFeeRecipientTx(routerAddress: string): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'clear_fee_recipient', args: [] };
}

export function buildClaimFeesTx(
  routerAddress: string,
  beneficiary: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'claim_fees', args: [beneficiary] };
}

export function buildTransferWarpRouteOwnershipTx(
  routerAddress: string,
  newOwner: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'transfer_ownership', args: [newOwner] };
}

export function buildAcceptWarpRouteOwnershipTx(
  routerAddress: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'accept_ownership', args: [] };
}

export function buildRescueNativeTx(
  routerAddress: string,
  amount: bigint,
  recipient: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'rescue_native', args: [amount.toString(), recipient] };
}

export function buildRescueTokenTx(
  routerAddress: string,
  tokenAddress: string,
  amount: bigint,
  recipient: string,
): AeternityTransaction {
  return { contractId: routerAddress, entrypoint: 'rescue_token', args: [tokenAddress, amount.toString(), recipient] };
}
