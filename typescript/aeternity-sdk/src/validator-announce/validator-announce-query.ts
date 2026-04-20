import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { VALIDATOR_ANNOUNCE_ACI } from '../aci/index.js';

export async function getAnnouncedValidators(
  sdk: AeSdk,
  validatorAnnounceAddress: string,
): Promise<string[]> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: VALIDATOR_ANNOUNCE_ACI,
    address: validatorAnnounceAddress as `ct_${string}`,
  });

  const result = await contract.get_announced_validators();
  return result.decodedResult;
}

export async function getAnnouncedStorageLocations(
  sdk: AeSdk,
  validatorAnnounceAddress: string,
  validators: string[],
): Promise<string[][]> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: VALIDATOR_ANNOUNCE_ACI,
    address: validatorAnnounceAddress as `ct_${string}`,
  });

  const result = await contract.get_announced_storage_locations(validators);
  return result.decodedResult;
}
