import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import {
  CHECKPOINT_FRAUD_PROOFS_ACI,
  ATTRIBUTE_CHECKPOINT_FRAUD_ACI,
} from '../aci/index.js';

export async function isPrematureCheckpoint(
  sdk: AeSdk,
  address: string,
  checkpoint: { root: string; index: number; merkle_tree: string },
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [CHECKPOINT_FRAUD_PROOFS_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.is_premature(checkpoint);
  return Boolean(result.decodedResult);
}

export async function isFraudulentMessageId(
  sdk: AeSdk,
  address: string,
  checkpoint: { root: string; index: number; merkle_tree: string },
  proof: string[],
  actualMessageId: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [CHECKPOINT_FRAUD_PROOFS_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.is_fraudulent_message_id(
    checkpoint,
    proof,
    actualMessageId,
  );
  return Boolean(result.decodedResult);
}

export async function isFraudulentRoot(
  sdk: AeSdk,
  address: string,
  checkpoint: { root: string; index: number; merkle_tree: string },
  proof: string[],
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [CHECKPOINT_FRAUD_PROOFS_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.is_fraudulent_root(checkpoint, proof);
  return Boolean(result.decodedResult);
}

export async function getFraudAttribution(
  sdk: AeSdk,
  address: string,
  checkpoint: object,
  signature: string,
): Promise<{ signer: string; fraudType: number; timestamp: bigint } | null> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [ATTRIBUTE_CHECKPOINT_FRAUD_ACI],
    address: address as `ct_${string}`,
  });
  const result = await contract.get_attribution(checkpoint, signature);
  const decoded = result.decodedResult;
  if (!decoded) return null;

  return {
    signer: decoded.signer,
    fraudType: Number(decoded.fraud_type),
    timestamp: BigInt(decoded.timestamp),
  };
}
