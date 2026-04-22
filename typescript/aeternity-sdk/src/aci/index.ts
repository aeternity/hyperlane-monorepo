export const MAILBOX_ACI = {
  contract: {
    name: 'Mailbox',
    kind: 'contract_main',
    payable: true,
    typedefs: [],
    functions: [
      {
        name: 'version',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'local_domain',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'deployed_block',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'nonce',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'latest_dispatched_id',
        arguments: [],
        returns: { bytes: 32 },
        stateful: false,
        payable: false,
      },
      {
        name: 'default_ism',
        arguments: [],
        returns: { option: ['IInterchainSecurityModule'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'default_hook',
        arguments: [],
        returns: { option: ['IPostDispatchHook'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'required_hook',
        arguments: [],
        returns: { option: ['IPostDispatchHook'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'delivered',
        arguments: [{ name: 'id', type: { bytes: 32 } }],
        returns: 'bool',
        stateful: false,
        payable: false,
      },
      {
        name: 'owner',
        arguments: [],
        returns: 'address',
        stateful: false,
        payable: false,
      },
      {
        name: 'dispatch',
        arguments: [
          { name: 'destination_domain', type: 'int' },
          { name: 'recipient_address', type: { bytes: 32 } },
          { name: 'message_body', type: { bytes: 'any' } },
          { name: 'hook_metadata', type: { option: [{ bytes: 'any' }] } },
          { name: 'custom_hook', type: { option: ['IPostDispatchHook'] } },
        ],
        returns: { bytes: 32 },
        stateful: true,
        payable: true,
      },
      {
        name: 'quote_dispatch',
        arguments: [
          { name: 'destination_domain', type: 'int' },
          { name: 'recipient_address', type: { bytes: 32 } },
          { name: 'message_body', type: { bytes: 'any' } },
          { name: 'hook_metadata', type: { option: [{ bytes: 'any' }] } },
          { name: 'custom_hook', type: { option: ['IPostDispatchHook'] } },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'process',
        arguments: [
          { name: 'metadata', type: { bytes: 'any' } },
          { name: 'message', type: { bytes: 'any' } },
          { name: 'recipient_addr', type: 'address' },
          { name: 'body_recipient_addr', type: 'address' },
        ],
        returns: 'unit',
        stateful: true,
        payable: true,
      },
    ],
  },
};

export const MERKLE_TREE_HOOK_ACI = {
  contract: {
    name: 'MerkleTreeHook',
    kind: 'contract_main',
    payable: true,
    typedefs: [],
    functions: [
      {
        name: 'hook_type',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'count',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'root',
        arguments: [],
        returns: { bytes: 32 },
        stateful: false,
        payable: false,
      },
      {
        name: 'latest_checkpoint',
        arguments: [],
        returns: { tuple: [{ bytes: 32 }, 'int'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'get_mailbox',
        arguments: [],
        returns: 'address',
        stateful: false,
        payable: false,
      },
      {
        name: 'quote_dispatch',
        arguments: [
          { name: '_metadata', type: { bytes: 'any' } },
          { name: '_message', type: { bytes: 'any' } },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
    ],
  },
};

export const MULTISIG_ISM_ACI = {
  contract: {
    name: 'MessageIdMultisigIsm',
    kind: 'contract_main',
    payable: false,
    typedefs: [],
    functions: [
      {
        name: 'module_type',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'get_validators',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: { list: [{ bytes: 20 }] },
        stateful: false,
        payable: false,
      },
      {
        name: 'get_threshold',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'validators_and_threshold',
        arguments: [{ name: 'message', type: { bytes: 'any' } }],
        returns: { tuple: [{ list: [{ bytes: 20 }] }, 'int'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'verify',
        arguments: [
          { name: 'metadata', type: { bytes: 'any' } },
          { name: 'message', type: { bytes: 'any' } },
        ],
        returns: 'bool',
        stateful: false,
        payable: false,
      },
      {
        name: 'set_validators_and_threshold',
        arguments: [
          { name: 'domain', type: 'int' },
          { name: 'vals', type: { list: [{ bytes: 20 }] } },
          { name: 'threshold', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
    ],
  },
};

export const VALIDATOR_ANNOUNCE_ACI = {
  contract: {
    name: 'ValidatorAnnounce',
    kind: 'contract_main',
    payable: false,
    typedefs: [],
    functions: [
      {
        name: 'get_announced_validators',
        arguments: [],
        returns: { list: [{ bytes: 20 }] },
        stateful: false,
        payable: false,
      },
      {
        name: 'get_announced_storage_locations',
        arguments: [{ name: 'validators', type: { list: [{ bytes: 20 }] } }],
        returns: { list: [{ list: ['string'] }] },
        stateful: false,
        payable: false,
      },
      {
        name: 'announce',
        arguments: [
          { name: 'validator', type: { bytes: 20 } },
          { name: 'storage_location', type: 'string' },
          { name: 'signature', type: { bytes: 65 } },
        ],
        returns: 'bool',
        stateful: true,
        payable: false,
      },
      {
        name: 'get_announcement_digest',
        arguments: [{ name: 'location', type: 'string' }],
        returns: { bytes: 32 },
        stateful: false,
        payable: false,
      },
    ],
  },
};

export const NOOP_HOOK_ACI = {
  contract: {
    name: 'NoopHook',
    kind: 'contract_main',
    payable: true,
    typedefs: [],
    functions: [
      {
        name: 'hook_type',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'quote_dispatch',
        arguments: [
          { name: '_metadata', type: { bytes: 'any' } },
          { name: '_message', type: { bytes: 'any' } },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
    ],
  },
};

export const AEX9_ACI = {
  contract: {
    name: 'MintableAEX9',
    kind: 'contract_main',
    payable: false,
    typedefs: [],
    functions: [
      {
        name: 'name',
        arguments: [],
        returns: 'string',
        stateful: false,
        payable: false,
      },
      {
        name: 'symbol',
        arguments: [],
        returns: 'string',
        stateful: false,
        payable: false,
      },
      {
        name: 'decimals',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'total_supply',
        arguments: [],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'balance',
        arguments: [{ name: 'owner', type: 'address' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'allowance',
        arguments: [
          { name: 'owner', type: 'address' },
          { name: 'spender', type: 'address' },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'transfer',
        arguments: [
          { name: 'to', type: 'address' },
          { name: 'value', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
      {
        name: 'create_allowance',
        arguments: [
          { name: 'spender', type: 'address' },
          { name: 'value', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
      {
        name: 'mint',
        arguments: [
          { name: 'to', type: 'address' },
          { name: 'amount', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
      {
        name: 'burn',
        arguments: [
          { name: 'from', type: 'address' },
          { name: 'amount', type: 'int' },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
    ],
  },
};

export const WARP_ROUTER_ACI = {
  contract: {
    name: 'WarpRouter',
    kind: 'contract_main',
    payable: true,
    typedefs: [],
    functions: [
      {
        name: 'transfer_remote',
        arguments: [
          { name: 'destination', type: 'int' },
          { name: 'recipient', type: { bytes: 32 } },
          { name: 'amount', type: 'int' },
        ],
        returns: { bytes: 32 },
        stateful: true,
        payable: true,
      },
      {
        name: 'quote_transfer_remote',
        arguments: [
          { name: 'destination', type: 'int' },
          { name: 'recipient', type: { bytes: 32 } },
          { name: 'amount', type: 'int' },
        ],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'get_remote_router',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: { option: [{ bytes: 32 }] },
        stateful: false,
        payable: false,
      },
      {
        name: 'get_destination_gas',
        arguments: [{ name: 'domain', type: 'int' }],
        returns: 'int',
        stateful: false,
        payable: false,
      },
      {
        name: 'get_decimal_scaling',
        arguments: [],
        returns: { tuple: ['int', 'int'] },
        stateful: false,
        payable: false,
      },
      {
        name: 'enroll_remote_router',
        arguments: [
          { name: 'domain', type: 'int' },
          { name: 'router', type: { bytes: 32 } },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
      {
        name: 'enroll_remote_routers',
        arguments: [
          {
            name: 'entries',
            type: { list: [{ tuple: ['int', { bytes: 32 }] }] },
          },
        ],
        returns: 'unit',
        stateful: true,
        payable: false,
      },
    ],
  },
};
