import { expect } from 'chai';

import {
  getHookType,
  getMerkleTreeHookConfig,
  getHookQuoteDispatch,
  getDomainRoutingHookConfig,
  isPausableHookPaused,
} from './hook-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Hook query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getHookType', () => {
    it('returns the hook type as a number', async () => {
      mockContractInitialize({
        hook_type: mockMethod(3),
      });
      const result = await getHookType(createMockSdk(), 'ct_testHook');
      expect(result).to.equal(3);
    });

    it('converts bigint results to number', async () => {
      mockContractInitialize({
        hook_type: mockMethod(BigInt(5)),
      });
      const result = await getHookType(createMockSdk(), 'ct_testHook');
      expect(result).to.equal(5);
    });
  });

  describe('getMerkleTreeHookConfig', () => {
    it('returns the full merkle tree hook configuration', async () => {
      const mockRoot =
        '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';
      const checkpointRoot =
        '0x1111111111111111111111111111111111111111111111111111111111111111';

      mockContractInitialize({
        count: mockMethod(42),
        root: mockMethod(mockRoot),
        latest_checkpoint: mockMethod([checkpointRoot, BigInt(41)]),
        get_mailbox: mockMethod('ct_mailboxAddr'),
      });

      const config = await getMerkleTreeHookConfig(
        createMockSdk(),
        'ct_testHook',
      );

      expect(config.count).to.equal(42);
      expect(config.root).to.equal(mockRoot);
      expect(config.latestCheckpoint.root).to.equal(checkpointRoot);
      expect(config.latestCheckpoint.index).to.equal(41);
      expect(config.mailbox).to.equal('ct_mailboxAddr');
    });

    it('handles zero count and empty root', async () => {
      const zeroRoot =
        '0x0000000000000000000000000000000000000000000000000000000000000000';

      mockContractInitialize({
        count: mockMethod(0),
        root: mockMethod(zeroRoot),
        latest_checkpoint: mockMethod([zeroRoot, BigInt(0)]),
        get_mailbox: mockMethod('ct_mailboxAddr'),
      });

      const config = await getMerkleTreeHookConfig(
        createMockSdk(),
        'ct_testHook',
      );

      expect(config.count).to.equal(0);
      expect(config.latestCheckpoint.index).to.equal(0);
    });
  });

  describe('getHookQuoteDispatch', () => {
    it('returns the quote dispatch amount as bigint', async () => {
      mockContractInitialize({
        hook_type: mockMethod(4),
        quote_dispatch: mockMethod(BigInt(1000)),
      });

      const result = await getHookQuoteDispatch(createMockSdk(), 'ct_testHook');
      expect(result).to.equal(BigInt(1000));
    });

    it('returns zero for free hooks', async () => {
      mockContractInitialize({
        hook_type: mockMethod(13),
        quote_dispatch: mockMethod(0),
      });

      const result = await getHookQuoteDispatch(createMockSdk(), 'ct_testHook');
      expect(result).to.equal(BigInt(0));
    });
  });

  describe('getDomainRoutingHookConfig', () => {
    it('returns owner and domains', async () => {
      mockContractInitialize({
        get_owner: mockMethod('ak_ownerAddr'),
        get_domains: mockMethod([457, 11155111]),
      });

      const config = await getDomainRoutingHookConfig(
        createMockSdk(),
        'ct_testHook',
      );

      expect(config.owner).to.equal('ak_ownerAddr');
      expect(config.domains).to.deep.equal([457, 11155111]);
    });

    it('handles empty domain list', async () => {
      mockContractInitialize({
        get_owner: mockMethod('ak_owner'),
        get_domains: mockMethod([]),
      });

      const config = await getDomainRoutingHookConfig(
        createMockSdk(),
        'ct_testHook',
      );

      expect(config.domains).to.be.empty;
    });
  });

  describe('isPausableHookPaused', () => {
    it('returns true when hook is paused', async () => {
      mockContractInitialize({
        is_paused: mockMethod(true),
      });

      const result = await isPausableHookPaused(createMockSdk(), 'ct_testHook');
      expect(result).to.be.true;
    });

    it('returns false when hook is not paused', async () => {
      mockContractInitialize({
        is_paused: mockMethod(false),
      });

      const result = await isPausableHookPaused(createMockSdk(), 'ct_testHook');
      expect(result).to.be.false;
    });
  });
});
