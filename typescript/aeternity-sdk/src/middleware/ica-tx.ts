import { AeternityTransaction } from '../utils/types.js';

export function buildCallRemoteAeTx(
  address: string,
  destination: number,
  recipients: Array<[string, bigint]>,
): AeternityTransaction {
  const encodedRecipients = recipients.map(([addr, amount]) => [
    addr,
    amount.toString(),
  ]);
  return {
    contractId: address,
    entrypoint: 'call_remote_ae',
    args: [destination, encodedRecipients],
  };
}

export function buildEnrollIcaRemoteRouterTx(
  address: string,
  domain: number,
  router: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'enroll_remote_router',
    args: [domain, router],
  };
}
