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

export interface TransferQuote {
  dispatchCost: bigint;
  feeAmount: bigint;
  totalToken: bigint;
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
  MERKLE_ROOT_MULTISIG = 'MerkleRootMultisigIsm',
  WEIGHTED_MULTISIG = 'WeightedMultisigIsm',
  DOMAIN_ROUTING = 'DomainRoutingIsm',
  INCREMENTAL_DOMAIN_ROUTING = 'IncrementalDomainRoutingIsm',
  AGGREGATION = 'AggregationIsm',
  PAUSABLE = 'PausableIsm',
  TRUSTED_RELAYER = 'TrustedRelayerIsm',
  RATE_LIMITED = 'RateLimitedIsm',
  TIMELOCK = 'TimelockIsm',
  NOOP = 'NoopIsm',
  AMOUNT_ROUTING = 'AmountRoutingIsm',
  TIMELOCK_DOMAIN_ROUTING = 'TimelockDomainRoutingIsm',
}

export enum AeternityHookTypes {
  MERKLE_TREE = 'MerkleTreeHook',
  NOOP = 'NoopHook',
  IGP = 'InterchainGasPaymaster',
  PROTOCOL_FEE = 'ProtocolFee',
  DOMAIN_ROUTING = 'DomainRoutingHook',
  FALLBACK_DOMAIN_ROUTING = 'FallbackDomainRoutingHook',
  PAUSABLE = 'PausableHook',
  STATIC_AGGREGATION = 'StaticAggregationHook',
  RATE_LIMITED = 'RateLimitedHook',
  AMOUNT_ROUTING = 'AmountRoutingHook',
  DESTINATION_RECIPIENT_ROUTING = 'DestinationRecipientRoutingHook',
  OFFCHAIN_QUOTED_IGP = 'OffchainQuotedIGP',
}

export interface TransferQuote {
  dispatchCost: bigint;
  feeAmount: bigint;
  totalToken: bigint;
}

export const HOOK_TYPE_NUMBERS = {
  NOOP: 0,
  MERKLE_TREE: 1,
  IGP: 4,
  PROTOCOL_FEE: 5,
  DOMAIN_ROUTING: 6,
  FALLBACK_DOMAIN_ROUTING: 7,
  PAUSABLE: 8,
  STATIC_AGGREGATION: 9,
  RATE_LIMITED: 10,
} as const;

export const ISM_MODULE_TYPE_NUMBERS = {
  UNUSED: 0,
  ROUTING: 1,
  AGGREGATION: 2,
  LEGACY_MULTISIG: 3,
  MERKLE_ROOT_MULTISIG: 4,
  MESSAGE_ID_MULTISIG: 5,
  NULL: 6,
  CCIP_READ: 7,
} as const;

export enum AeternityEventTypes {
  OWNERSHIP_TRANSFER_STARTED = 'OwnershipTransferStarted',
  OWNERSHIP_TRANSFERRED = 'OwnershipTransferred',
  PAUSED = 'Paused',
  UNPAUSED = 'Unpaused',
  FEE_RECIPIENT_SET = 'FeeRecipientSet',
  FEE_CHARGED = 'FeeCharged',
  FEE_CLAIMED = 'FeeClaimed',
  NATIVE_RESCUED = 'NativeRescued',
  RATE_LIMIT_SET = 'RateLimitSet',
  CONSUMED_FILLED_LEVEL = 'ConsumedFilledLevel',
  ISM_CHANGE_QUEUED = 'IsmChangeQueued',
  ISM_CHANGE_EXECUTED = 'IsmChangeExecuted',
  ISM_CHANGE_CANCELLED = 'IsmChangeCancelled',
  OPERATION_SCHEDULED = 'OperationScheduled',
  OPERATION_EXECUTED = 'OperationExecuted',
  OPERATION_CANCELLED = 'OperationCancelled',
  VALIDATOR_STAKED = 'ValidatorStaked',
  VALIDATOR_SLASHED = 'ValidatorSlashed',
  UNSTAKE_INITIATED = 'UnstakeInitiated',
  UNSTAKE_COMPLETED = 'UnstakeCompleted',
  FRAUD_ATTRIBUTED = 'FraudAttributed',
  FRAUD_SLASH_EXECUTED = 'FraudSlashExecuted',
}
