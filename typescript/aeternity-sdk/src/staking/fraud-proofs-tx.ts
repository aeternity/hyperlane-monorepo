import { AeternityTransaction } from '../utils/types.js';

export function buildAttributePrematureTx(
  address: string,
  checkpoint: object,
  signature: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'attribute_premature',
    args: [checkpoint, signature],
  };
}

export function buildAttributeMessageIdTx(
  address: string,
  checkpoint: object,
  proof: string[],
  actualMsgId: string,
  signature: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'attribute_message_id',
    args: [checkpoint, proof, actualMsgId, signature],
  };
}

export function buildAttributeRootTx(
  address: string,
  checkpoint: object,
  proof: string[],
  signature: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'attribute_root',
    args: [checkpoint, proof, signature],
  };
}

export function buildWhitelistMerkleTreeTx(
  address: string,
  merkleTree: string,
): AeternityTransaction {
  return {
    contractId: address,
    entrypoint: 'whitelist_merkle_tree',
    args: [merkleTree],
  };
}
