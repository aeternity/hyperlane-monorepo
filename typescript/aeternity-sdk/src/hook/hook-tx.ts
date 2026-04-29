import { AeternityTransaction } from '../utils/types.js';

export function buildSetHookTx(
  mailboxAddress: string,
  hookAddress: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'set_default_hook',
    args: [hookAddress],
  };
}

export function buildSetRequiredHookTx(
  mailboxAddress: string,
  hookAddress: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'set_required_hook',
    args: [hookAddress],
  };
}

export function buildInitializeMailboxTx(
  mailboxAddress: string,
  defaultIsm: string,
  defaultHook: string,
  requiredHook: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'initialize',
    args: [defaultIsm, defaultHook, requiredHook],
  };
}

export function buildRenounceOwnershipTx(
  contractAddress: string,
): AeternityTransaction {
  return {
    contractId: contractAddress,
    entrypoint: 'renounce_ownership',
    args: [],
  };
}

export function buildSetOwnIsmTx(
  mailboxAddress: string,
  ism: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'set_own_ism',
    args: [ism],
  };
}
