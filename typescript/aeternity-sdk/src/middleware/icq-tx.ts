import { AeternityTransaction } from '../utils/types.js';

export function buildRegisterViewTargetTx(
  address: string,
  target: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'register_view_target',
    args: [target],
  };
}

export function buildEnrollIcqRemoteRouterTx(
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
