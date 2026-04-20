import { expect } from 'chai';

import {
  getMailboxState,
  isMessageDelivered,
  quoteDispatch,
} from './mailbox-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Mailbox query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getMailboxState', () => {
    it('returns the full mailbox state', async () => {
      const latestId =
        '0xdeadbeef12345678deadbeef12345678deadbeef12345678deadbeef12345678';

      mockContractInitialize({
        local_domain: mockMethod(457),
        nonce: mockMethod(10),
        default_ism: mockMethod('ct_ismAddr'),
        default_hook: mockMethod('ct_hookAddr'),
        required_hook: mockMethod('ct_reqHookAddr'),
        owner: mockMethod('ak_ownerAddr'),
        deployed_block: mockMethod(100),
        latest_dispatched_id: mockMethod(latestId),
      });

      const state = await getMailboxState(
        createMockSdk(),
        'ct_mailbox',
      );

      expect(state.localDomain).to.equal(457);
      expect(state.nonce).to.equal(10);
      expect(state.defaultIsm).to.equal('ct_ismAddr');
      expect(state.defaultHook).to.equal('ct_hookAddr');
      expect(state.requiredHook).to.equal('ct_reqHookAddr');
      expect(state.owner).to.equal('ak_ownerAddr');
      expect(state.deployedBlock).to.equal(100);
      expect(state.latestDispatchedId).to.equal(latestId);
    });

    it('handles undefined optional fields', async () => {
      mockContractInitialize({
        local_domain: mockMethod(457),
        nonce: mockMethod(0),
        default_ism: mockMethod(undefined),
        default_hook: mockMethod(undefined),
        required_hook: mockMethod(undefined),
        owner: mockMethod('ak_owner'),
        deployed_block: mockMethod(1),
        latest_dispatched_id: mockMethod('0x00'),
      });

      const state = await getMailboxState(
        createMockSdk(),
        'ct_mailbox',
      );

      expect(state.defaultIsm).to.equal('');
      expect(state.defaultHook).to.equal('');
      expect(state.requiredHook).to.equal('');
    });
  });

  describe('isMessageDelivered', () => {
    it('returns true when message has been delivered', async () => {
      mockContractInitialize({
        delivered: mockMethod(true),
      });

      const result = await isMessageDelivered(
        createMockSdk(),
        'ct_mailbox',
        '0x1234',
      );
      expect(result).to.be.true;
    });

    it('returns false when message has not been delivered', async () => {
      mockContractInitialize({
        delivered: mockMethod(false),
      });

      const result = await isMessageDelivered(
        createMockSdk(),
        'ct_mailbox',
        '0x5678',
      );
      expect(result).to.be.false;
    });
  });

  describe('quoteDispatch', () => {
    it('returns the dispatch fee as bigint', async () => {
      mockContractInitialize({
        quote_dispatch: mockMethod(BigInt(5000)),
      });

      const fee = await quoteDispatch(
        createMockSdk(),
        'ct_mailbox',
        11155111,
        '0xrecipient',
        new Uint8Array([1, 2, 3]),
      );

      expect(fee).to.equal(BigInt(5000));
    });

    it('returns zero when dispatch is free', async () => {
      mockContractInitialize({
        quote_dispatch: mockMethod(0),
      });

      const fee = await quoteDispatch(
        createMockSdk(),
        'ct_mailbox',
        457,
        '0xrecipient',
        new Uint8Array([]),
      );

      expect(fee).to.equal(BigInt(0));
    });
  });
});
