export interface AeternityTransaction {
  contractId: string;
  entrypoint: string;
  args: any[];
  options?: {
    amount?: bigint;
    gas?: number;
    gasPrice?: number;
  };
}

export interface AeternityReceipt {
  hash: string;
  blockHeight: number;
  blockHash: string;
  returnValue?: any;
  gasUsed: number;
  log: any[];
}

export enum AeternityIsmTypes {
  MESSAGE_ID_MULTISIG = 'MessageIdMultisigIsm',
  DOMAIN_ROUTING = 'DomainRoutingIsm',
}

export enum AeternityHookTypes {
  MERKLE_TREE = 'MerkleTreeHook',
  NOOP = 'NoopHook',
  IGP = 'InterchainGasPaymaster',
  PROTOCOL_FEE = 'ProtocolFee',
}
