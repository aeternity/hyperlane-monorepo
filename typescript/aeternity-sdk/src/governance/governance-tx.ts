import { AeternityTransaction } from '../utils/types.js';

export function buildScheduleOperationTx(
  address: string,
  target: string,
  callData: string,
  value: bigint,
  delay: bigint,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'schedule',
    args: [target, callData, value.toString(), delay.toString()],
  };
}

export function buildExecuteOperationTx(
  address: string,
  opId: string,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'execute', args: [opId] };
}

export function buildCancelOperationTx(
  address: string,
  opId: string,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'cancel', args: [opId] };
}

export function buildSetMinDelayTx(
  address: string,
  newDelay: bigint,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'set_min_delay',
    args: [newDelay.toString()],
  };
}

export function buildAddProposerTx(
  address: string,
  proposer: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'add_proposer',
    args: [proposer],
  };
}

export function buildRemoveProposerTx(
  address: string,
  proposer: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'remove_proposer',
    args: [proposer],
  };
}

export function buildAddExecutorTx(
  address: string,
  executor: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'add_executor',
    args: [executor],
  };
}

export function buildRemoveExecutorTx(
  address: string,
  executor: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'remove_executor',
    args: [executor],
  };
}

export function buildSubmitMultiSigTx(
  address: string,
  target: string,
  value: bigint,
  data: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'submit',
    args: [target, value.toString(), data],
  };
}

export function buildConfirmMultiSigTx(
  address: string,
  txId: number,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'confirm', args: [txId] };
}

export function buildRevokeMultiSigTx(
  address: string,
  txId: number,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'revoke', args: [txId] };
}

export function buildExecuteMultiSigTx(
  address: string,
  txId: number,
): AeternityTransaction {
  return { contractId: address, entrypoint: 'execute', args: [txId] };
}
