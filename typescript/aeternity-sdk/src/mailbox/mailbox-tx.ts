import type { AeternityTransaction } from '../utils/types.js';

export function buildSetMaxMessageBodyBytesTx(
  mailboxAddress: string,
  maxBytes: number,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'set_max_message_body_bytes',
    args: [maxBytes],
  };
}

export function buildTransferMailboxOwnershipTx(
  mailboxAddress: string,
  newOwner: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'transfer_ownership',
    args: [newOwner],
  };
}

export function buildAcceptMailboxOwnershipTx(
  mailboxAddress: string,
): AeternityTransaction {
  return {
    contractId: mailboxAddress,
    entrypoint: 'accept_ownership',
    args: [],
  };
}
