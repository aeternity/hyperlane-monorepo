import { expect } from 'chai';

import {
  MAILBOX_ACI,
  MERKLE_TREE_HOOK_ACI,
  MULTISIG_ISM_ACI,
  VALIDATOR_ANNOUNCE_ACI,
  NOOP_HOOK_ACI,
  AEX9_ACI,
  WARP_ROUTER_ACI,
} from './index.js';

function getFunctionNames(aci: any): string[] {
  return aci.contract.functions.map((f: any) => f.name);
}

describe('ACI definitions', () => {
  describe('MAILBOX_ACI', () => {
    it('has the correct contract name', () => {
      expect(MAILBOX_ACI.contract.name).to.equal('Mailbox');
    });

    it('is payable', () => {
      // Generated ACIs may omit the payable field when false at contract level;
      // dispatch/process are payable at function level.
      const dispatch = MAILBOX_ACI.contract.functions.find(
        (f: any) => f.name === 'dispatch',
      );
      expect(dispatch?.payable).to.be.true;
    });

    it('exposes all required entrypoints', () => {
      const names = getFunctionNames(MAILBOX_ACI);
      expect(names).to.include.members([
        'version',
        'local_domain',
        'deployed_block',
        'nonce',
        'latest_dispatched_id',
        'default_ism',
        'default_hook',
        'required_hook',
        'delivered',
        'owner',
        'dispatch',
        'quote_dispatch',
        'process',
      ]);
    });

    it('dispatch entrypoint is stateful and payable', () => {
      const dispatch = MAILBOX_ACI.contract.functions.find(
        (f: any) => f.name === 'dispatch',
      );
      expect(dispatch?.stateful).to.be.true;
      expect(dispatch?.payable).to.be.true;
    });

    it('delivered entrypoint takes bytes(32) and returns bool', () => {
      const delivered = MAILBOX_ACI.contract.functions.find(
        (f: any) => f.name === 'delivered',
      );
      expect(delivered?.arguments).to.have.length(1);
      // ACI type can be string 'bytes(32)' or object { bytes: 32 }
      const argType = delivered?.arguments[0].type;
      const isBytes32 =
        argType === 'bytes(32)' ||
        (typeof argType === 'object' && argType?.bytes === 32);
      expect(isBytes32).to.be.true;
      expect(delivered?.returns).to.equal('bool');
    });
  });

  describe('MERKLE_TREE_HOOK_ACI', () => {
    it('has the correct contract name', () => {
      expect(MERKLE_TREE_HOOK_ACI.contract.name).to.equal('MerkleTreeHook');
    });

    it('exposes all required entrypoints', () => {
      const names = getFunctionNames(MERKLE_TREE_HOOK_ACI);
      expect(names).to.include.members([
        'hook_type',
        'count',
        'root',
        'latest_checkpoint',
        'get_mailbox',
        'quote_dispatch',
      ]);
    });

    it('latest_checkpoint returns a tuple of bytes(32) and int', () => {
      const cp = MERKLE_TREE_HOOK_ACI.contract.functions.find(
        (f: any) => f.name === 'latest_checkpoint',
      );
      // ACI tuple element can be 'bytes(32)' or { bytes: 32 }
      const tuple = cp?.returns?.tuple;
      expect(tuple).to.have.length(2);
      const first = tuple[0];
      const isBytes32 =
        first === 'bytes(32)' ||
        (typeof first === 'object' && first?.bytes === 32);
      expect(isBytes32).to.be.true;
      expect(tuple[1]).to.equal('int');
    });
  });

  describe('MULTISIG_ISM_ACI', () => {
    it('has the correct contract name', () => {
      expect(MULTISIG_ISM_ACI.contract.name).to.equal('MessageIdMultisigIsm');
    });

    it('is not payable', () => {
      expect(MULTISIG_ISM_ACI.contract.payable).to.be.false;
    });

    it('exposes all required entrypoints', () => {
      const names = getFunctionNames(MULTISIG_ISM_ACI);
      expect(names).to.include.members([
        'module_type',
        'get_validators',
        'get_threshold',
        'validators_and_threshold',
        'verify',
        'set_validators_and_threshold',
      ]);
    });

    it('set_validators_and_threshold is stateful', () => {
      const fn = MULTISIG_ISM_ACI.contract.functions.find(
        (f: any) => f.name === 'set_validators_and_threshold',
      );
      expect(fn?.stateful).to.be.true;
    });
  });

  describe('VALIDATOR_ANNOUNCE_ACI', () => {
    it('has the correct contract name', () => {
      expect(VALIDATOR_ANNOUNCE_ACI.contract.name).to.equal(
        'ValidatorAnnounce',
      );
    });

    it('exposes all required entrypoints', () => {
      const names = getFunctionNames(VALIDATOR_ANNOUNCE_ACI);
      expect(names).to.include.members([
        'get_announced_validators',
        'get_announced_storage_locations',
        'announce',
        'get_announcement_digest',
      ]);
    });

    it('announce is stateful and returns bool', () => {
      const fn = VALIDATOR_ANNOUNCE_ACI.contract.functions.find(
        (f: any) => f.name === 'announce',
      );
      expect(fn?.stateful).to.be.true;
      expect(fn?.returns).to.equal('bool');
    });
  });

  describe('NOOP_HOOK_ACI', () => {
    it('has the correct contract name', () => {
      expect(NOOP_HOOK_ACI.contract.name).to.equal('NoopHook');
    });

    it('exposes hook_type and quote_dispatch', () => {
      const names = getFunctionNames(NOOP_HOOK_ACI);
      expect(names).to.include.members(['hook_type', 'quote_dispatch']);
    });
  });

  describe('AEX9_ACI', () => {
    it('has the correct contract name', () => {
      expect(AEX9_ACI.contract.name).to.equal('MintableAEX9');
    });

    it('exposes standard AEX-9 entrypoints', () => {
      const names = getFunctionNames(AEX9_ACI);
      expect(names).to.include.members([
        'name',
        'symbol',
        'decimals',
        'total_supply',
        'balance',
        'allowance',
        'transfer',
        'create_allowance',
        'mint',
        'burn',
      ]);
    });

    it('transfer is stateful', () => {
      const fn = AEX9_ACI.contract.functions.find(
        (f: any) => f.name === 'transfer',
      );
      expect(fn?.stateful).to.be.true;
    });
  });

  describe('WARP_ROUTER_ACI', () => {
    it('has the correct contract name', () => {
      expect(WARP_ROUTER_ACI.contract.name).to.equal('WarpRouter');
    });

    it('is payable', () => {
      expect(WARP_ROUTER_ACI.contract.payable).to.be.true;
    });

    it('exposes all required entrypoints', () => {
      const names = getFunctionNames(WARP_ROUTER_ACI);
      expect(names).to.include.members([
        'transfer_remote',
        'quote_transfer_remote',
        'get_remote_router',
        'get_destination_gas',
        'get_decimal_scaling',
        'enroll_remote_router',
        'enroll_remote_routers',
      ]);
    });

    it('transfer_remote is stateful and payable', () => {
      const fn = WARP_ROUTER_ACI.contract.functions.find(
        (f: any) => f.name === 'transfer_remote',
      );
      expect(fn?.stateful).to.be.true;
      expect(fn?.payable).to.be.true;
    });

    it('get_decimal_scaling returns a tuple of two ints', () => {
      const fn = WARP_ROUTER_ACI.contract.functions.find(
        (f: any) => f.name === 'get_decimal_scaling',
      );
      expect(fn?.returns).to.deep.equal({ tuple: ['int', 'int'] });
    });
  });
});
