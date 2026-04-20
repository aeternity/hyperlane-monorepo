import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { MERKLE_TREE_HOOK_ACI, NOOP_HOOK_ACI } from '../aci/index.js';

export async function getHookType(
  sdk: AeSdk,
  hookAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: MERKLE_TREE_HOOK_ACI,
    address: hookAddress as `ct_${string}`,
  });

  const result = await contract.hook_type();
  return Number(result.decodedResult);
}

export async function getMerkleTreeHookConfig(
  sdk: AeSdk,
  hookAddress: string,
): Promise<{
  count: number;
  root: string;
  latestCheckpoint: { root: string; index: number };
  mailbox: string;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: MERKLE_TREE_HOOK_ACI,
    address: hookAddress as `ct_${string}`,
  });

  const [countResult, rootResult, checkpointResult, mailboxResult] =
    await Promise.all([
      contract.count(),
      contract.root(),
      contract.latest_checkpoint(),
      contract.get_mailbox(),
    ]);

  const checkpoint = checkpointResult.decodedResult;

  return {
    count: Number(countResult.decodedResult),
    root: rootResult.decodedResult,
    latestCheckpoint: {
      root: checkpoint[0],
      index: Number(checkpoint[1]),
    },
    mailbox: mailboxResult.decodedResult,
  };
}

export async function getHookQuoteDispatch(
  sdk: AeSdk,
  hookAddress: string,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: NOOP_HOOK_ACI,
    address: hookAddress as `ct_${string}`,
  });

  const result = await contract.quote_dispatch(
    new Uint8Array(0),
    new Uint8Array(0),
  );
  return BigInt(result.decodedResult);
}
