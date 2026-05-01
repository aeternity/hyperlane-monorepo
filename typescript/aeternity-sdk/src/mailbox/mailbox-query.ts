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
  maxMessageBodyBytes: number;
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
    maxMessageBodyBytesResult,
  ] = await Promise.all([
    contract.local_domain(),
    contract.nonce(),
    contract.default_ism(),
    contract.default_hook(),
    contract.required_hook(),
    contract.owner(),
    contract.deployed_block(),
    contract.latest_dispatched_id(),
    contract.get_max_message_body_bytes(),
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
    maxMessageBodyBytes: Number(maxMessageBodyBytesResult.decodedResult),
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

export async function getMessageProcessor(
  sdk: AeSdk,
  mailboxAddress: string,
  messageId: string,
): Promise<string | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const result = await contract.processor(messageId);
  return result.decodedResult ?? null;
}

export async function getMessageProcessedAt(
  sdk: AeSdk,
  mailboxAddress: string,
  messageId: string,
): Promise<number | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const result = await contract.processed_at(messageId);
  return result.decodedResult != null ? Number(result.decodedResult) : null;
}

export async function getRecipientIsm(
  sdk: AeSdk,
  mailboxAddress: string,
  recipientAddress: string,
): Promise<string> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });

  const result = await contract.recipient_ism_for(recipientAddress);
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

export async function getMaxMessageBodyBytes(
  sdk: AeSdk,
  mailboxAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [MAILBOX_ACI],
    address: mailboxAddress as `ct_${string}`,
  });
  const result = await contract.get_max_message_body_bytes();
  return Number(result.decodedResult);
}
