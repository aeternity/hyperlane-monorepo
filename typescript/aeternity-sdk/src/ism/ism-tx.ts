import { AeternityTransaction } from '../utils/types.js';

export function buildSetValidatorsAndThresholdTx(
  ismAddress: string,
  domain: number,
  validators: string[],
  threshold: number,
): AeternityTransaction {
  return {
    contractId: ismAddress,
    entrypoint: 'set_validators_and_threshold',
    args: [domain, validators, threshold],
  };
}
