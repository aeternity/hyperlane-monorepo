import { AeternityTransaction } from '../utils/types.js';

export function buildPayForGasTx(
  igpAddress: string,
  messageId: Uint8Array,
  destinationDomain: number,
  gasAmount: bigint,
  paymentAmount: bigint,
): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'pay_for_gas',
    args: [messageId, destinationDomain, gasAmount],
    options: {
      amount: paymentAmount,
    },
  };
}

export function buildSetDestinationGasOverheadTx(
  igpAddress: string,
  domain: number,
  overhead: bigint,
): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'set_destination_gas_overhead',
    args: [domain, overhead],
  };
}

export function buildSetBeneficiaryTx(
  igpAddress: string,
  beneficiary: string,
): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'set_beneficiary',
    args: [beneficiary],
  };
}

export function buildClaimTx(igpAddress: string): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'claim',
    args: [],
  };
}

export function buildSetDestinationGasOverheadBatchTx(
  igpAddress: string,
  entries: Array<{ domain: number; overhead: bigint }>,
): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'set_destination_gas_overheads',
    args: [entries.map((e) => [e.domain, e.overhead])],
  };
}

export function buildSetOracleTx(
  igpAddress: string,
  oracle: string,
): AeternityTransaction {
  return {
    contractId: igpAddress,
    entrypoint: 'set_oracle',
    args: [oracle],
  };
}
