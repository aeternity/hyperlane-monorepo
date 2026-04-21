import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { MAILBOX_ACI } from '../aci/index.js';

export async function getMailboxState(
  sdk: AeSdk,
  mailboxAddress: string,
): Promise<{
  localDomain: number;
  nonce: number;
  defaultIsm: string;
  defaultHook: string;
  requiredHook: string;
  owner: string;
  deployedBlock: number;
  latestDispatchedId: string;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const [
    domainResult,
    nonceResult,
    ismResult,
    defaultHookResult,
    requiredHookResult,
    ownerResult,
    deployedBlockResult,
    latestIdResult,
  ] = await Promise.all([
    contract.local_domain(),
    contract.nonce(),
    contract.default_ism(),
    contract.default_hook(),
    contract.required_hook(),
    contract.owner(),
    contract.deployed_block(),
    contract.latest_dispatched_id(),
  ]);

  return {
    localDomain: Number(domainResult.decodedResult),
    nonce: Number(nonceResult.decodedResult),
    defaultIsm: ismResult.decodedResult?.toString() ?? '',
    defaultHook: defaultHookResult.decodedResult?.toString() ?? '',
    requiredHook: requiredHookResult.decodedResult?.toString() ?? '',
    owner: ownerResult.decodedResult,
    deployedBlock: Number(deployedBlockResult.decodedResult),
    latestDispatchedId: latestIdResult.decodedResult,
  };
}

export async function isMessageDelivered(
  sdk: AeSdk,
  mailboxAddress: string,
  messageId: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const result = await contract.delivered(messageId);
  return result.decodedResult;
}

export async function quoteDispatch(
  sdk: AeSdk,
  mailboxAddress: string,
  destinationDomain: number,
  recipientAddress: string,
  messageBody: Uint8Array,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const result = await contract.quote_dispatch(
    destinationDomain,
    recipientAddress,
    messageBody,
    undefined,
    undefined,
  );
  return BigInt(result.decodedResult);
}
