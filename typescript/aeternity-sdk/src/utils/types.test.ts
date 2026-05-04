import { expect } from 'chai';

import {
  AeternityIsmTypes,
  AeternityHookTypes,
  type AeternityTransaction,
  type AeternityReceipt,
} from './types.js';

describe('Aeternity types', () => {
  describe('AeternityIsmTypes', () => {
    it('defines MESSAGE_ID_MULTISIG', () => {
      expect(AeternityIsmTypes.MESSAGE_ID_MULTISIG).to.equal(
        'MessageIdMultisigIsm',
      );
    });

    it('defines MERKLE_ROOT_MULTISIG', () => {
      expect(AeternityIsmTypes.MERKLE_ROOT_MULTISIG).to.equal(
        'MerkleRootMultisigIsm',
      );
    });

    it('defines WEIGHTED_MULTISIG', () => {
      expect(AeternityIsmTypes.WEIGHTED_MULTISIG).to.equal(
        'WeightedMultisigIsm',
      );
    });

    it('defines DOMAIN_ROUTING', () => {
      expect(AeternityIsmTypes.DOMAIN_ROUTING).to.equal('DomainRoutingIsm');
    });

    it('defines INCREMENTAL_DOMAIN_ROUTING', () => {
      expect(AeternityIsmTypes.INCREMENTAL_DOMAIN_ROUTING).to.equal(
        'IncrementalDomainRoutingIsm',
      );
    });

    it('defines AGGREGATION', () => {
      expect(AeternityIsmTypes.AGGREGATION).to.equal('AggregationIsm');
    });

    it('defines PAUSABLE', () => {
      expect(AeternityIsmTypes.PAUSABLE).to.equal('PausableIsm');
    });

    it('defines TRUSTED_RELAYER', () => {
      expect(AeternityIsmTypes.TRUSTED_RELAYER).to.equal('TrustedRelayerIsm');
    });

    it('defines RATE_LIMITED', () => {
      expect(AeternityIsmTypes.RATE_LIMITED).to.equal('RateLimitedIsm');
    });

    it('defines TIMELOCK', () => {
      expect(AeternityIsmTypes.TIMELOCK).to.equal('TimelockIsm');
    });

    it('defines NOOP', () => {
      expect(AeternityIsmTypes.NOOP).to.equal('NoopIsm');
    });

    it('defines AMOUNT_ROUTING', () => {
      expect(AeternityIsmTypes.AMOUNT_ROUTING).to.equal('AmountRoutingIsm');
    });

    it('defines TIMELOCK_DOMAIN_ROUTING', () => {
      expect(AeternityIsmTypes.TIMELOCK_DOMAIN_ROUTING).to.equal(
        'TimelockDomainRoutingIsm',
      );
    });

    it('has exactly 13 members', () => {
      expect(Object.keys(AeternityIsmTypes)).to.have.length(13);
    });
  });

  describe('AeternityHookTypes', () => {
    it('defines MERKLE_TREE', () => {
      expect(AeternityHookTypes.MERKLE_TREE).to.equal('MerkleTreeHook');
    });

    it('defines NOOP', () => {
      expect(AeternityHookTypes.NOOP).to.equal('NoopHook');
    });

    it('defines IGP', () => {
      expect(AeternityHookTypes.IGP).to.equal('InterchainGasPaymaster');
    });

    it('defines PROTOCOL_FEE', () => {
      expect(AeternityHookTypes.PROTOCOL_FEE).to.equal('ProtocolFee');
    });

    it('defines DOMAIN_ROUTING', () => {
      expect(AeternityHookTypes.DOMAIN_ROUTING).to.equal('DomainRoutingHook');
    });

    it('defines FALLBACK_DOMAIN_ROUTING', () => {
      expect(AeternityHookTypes.FALLBACK_DOMAIN_ROUTING).to.equal(
        'FallbackDomainRoutingHook',
      );
    });

    it('defines PAUSABLE', () => {
      expect(AeternityHookTypes.PAUSABLE).to.equal('PausableHook');
    });

    it('defines STATIC_AGGREGATION', () => {
      expect(AeternityHookTypes.STATIC_AGGREGATION).to.equal(
        'StaticAggregationHook',
      );
    });

    it('defines RATE_LIMITED', () => {
      expect(AeternityHookTypes.RATE_LIMITED).to.equal('RateLimitedHook');
    });

    it('defines AMOUNT_ROUTING', () => {
      expect(AeternityHookTypes.AMOUNT_ROUTING).to.equal('AmountRoutingHook');
    });

    it('defines DESTINATION_RECIPIENT_ROUTING', () => {
      expect(AeternityHookTypes.DESTINATION_RECIPIENT_ROUTING).to.equal(
        'DestinationRecipientRoutingHook',
      );
    });

    it('defines OFFCHAIN_QUOTED_IGP', () => {
      expect(AeternityHookTypes.OFFCHAIN_QUOTED_IGP).to.equal(
        'OffchainQuotedIGP',
      );
    });

    it('has exactly 12 members', () => {
      expect(Object.keys(AeternityHookTypes)).to.have.length(12);
    });
  });

  describe('AeternityTransaction interface', () => {
    it('accepts a well-formed transaction object', () => {
      const tx: AeternityTransaction = {
        contractId: 'ct_test123',
        entrypoint: 'transfer',
        args: ['ak_recipient', 1000],
        options: {
          amount: BigInt(500),
          gas: 50000,
          gasPrice: 1000000000,
        },
      };

      expect(tx.contractId).to.equal('ct_test123');
      expect(tx.entrypoint).to.equal('transfer');
      expect(tx.args).to.have.length(2);
      expect(tx.options?.amount).to.equal(BigInt(500));
      expect(tx.options?.gas).to.equal(50000);
      expect(tx.options?.gasPrice).to.equal(1000000000);
    });

    it('accepts a transaction without options', () => {
      const tx: AeternityTransaction = {
        contractId: 'ct_test123',
        entrypoint: 'balance',
        args: ['ak_owner'],
      };

      expect(tx.contractId).to.equal('ct_test123');
      expect(tx.options).to.be.undefined;
    });
  });

  describe('AeternityReceipt interface', () => {
    it('accepts a well-formed receipt object', () => {
      const receipt: AeternityReceipt = {
        hash: 'th_abc123',
        blockHeight: 100,
        blockHash: 'kh_xyz789',
        returnValue: 42,
        gasUsed: 12345,
        log: [{ event: 'Transfer' }],
      };

      expect(receipt.hash).to.equal('th_abc123');
      expect(receipt.blockHeight).to.equal(100);
      expect(receipt.blockHash).to.equal('kh_xyz789');
      expect(receipt.returnValue).to.equal(42);
      expect(receipt.gasUsed).to.equal(12345);
      expect(receipt.log).to.have.length(1);
    });

    it('accepts a receipt without returnValue', () => {
      const receipt: AeternityReceipt = {
        hash: 'th_abc123',
        blockHeight: 100,
        blockHash: 'kh_xyz789',
        gasUsed: 0,
        log: [],
      };

      expect(receipt.returnValue).to.be.undefined;
      expect(receipt.log).to.be.empty;
    });
  });
});
