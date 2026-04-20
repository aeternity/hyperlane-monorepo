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
