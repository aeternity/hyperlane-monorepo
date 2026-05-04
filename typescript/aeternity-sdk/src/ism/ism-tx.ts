import { AeternityTransaction } from '../utils/types.js';

export function buildSetValidatorsAndThresholdTx(
  ismAddress: string,
  validators: string[],
  threshold: number,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: 'set_validators_and_threshold',
    args: [validators, threshold],
  };
}

export function buildRenounceIsmOwnershipTx(
  ismAddress: string,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: 'renounce_ownership',
    args: [],
  };
}

export function buildSetPauseTx(
  ismAddress: string,
  paused: boolean,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: paused ? 'pause' : 'unpause',
    args: [],
  };
}

export function buildSetDomainIsmTx(
  routingIsmAddress: string,
  domain: number,
  ismAddress: string,
): AeternityTransaction {
  return {
    contractId: routingIsmAddress,
    entrypoint: 'set_ism',
    args: [domain, ismAddress],
  };
}

export function buildSetAmountRoutingThresholdTx(
  ismAddress: string,
  threshold: bigint,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: 'set_threshold',
    args: [threshold.toString()],
  };
}

export function buildQueueSetIsmTx(
  timelockIsmAddress: string,
  domain: number,
  ismAddress: string,
): AeternityTransaction {
  return {
    contractId: timelockIsmAddress,
    entrypoint: 'queue_set_ism',
    args: [domain, ismAddress],
  };
}

export function buildExecuteIsmChangeTx(
  timelockIsmAddress: string,
  domain: number,
): AeternityTransaction {
  return {
    contractId: timelockIsmAddress,
    entrypoint: 'execute_change',
    args: [domain],
  };
}

export function buildCancelIsmChangeTx(
  timelockIsmAddress: string,
  domain: number,
): AeternityTransaction {
  return {
    contractId: timelockIsmAddress,
    entrypoint: 'cancel_change',
    args: [domain],
  };
}

export function buildSetDomainRoutingMailboxTx(
  ismAddress: string,
  mailboxAddress: string,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: 'set_mailbox',
    args: [mailboxAddress],
  };
}
