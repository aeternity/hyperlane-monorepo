import { expect } from 'chai';

import {
  getMailboxState,
  isMessageDelivered,
  quoteDispatch,
  getMessageProcessor,
  getMessageProcessedAt,
  getRecipientIsm,
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

  describe('getMessageProcessor', () => {
    it('returns the processor address when message was processed', async () => {
      mockContractInitialize({
        processor: mockMethod('ak_processor123'),
      });

      const result = await getMessageProcessor(
        createMockSdk(),
        'ct_mailbox',
        '0xabc',
      );
      expect(result).to.equal('ak_processor123');
    });

    it('returns null when message was not processed', async () => {
      mockContractInitialize({
        processor: mockMethod(undefined),
      });

      const result = await getMessageProcessor(
        createMockSdk(),
        'ct_mailbox',
        '0xdef',
      );
      expect(result).to.be.null;
    });
  });

  describe('getMessageProcessedAt', () => {
    it('returns the block number when message was processed', async () => {
      mockContractInitialize({
        processed_at: mockMethod(12345),
      });

      const result = await getMessageProcessedAt(
        createMockSdk(),
        'ct_mailbox',
        '0xabc',
      );
      expect(result).to.equal(12345);
    });

    it('returns null when message was not processed', async () => {
      mockContractInitialize({
        processed_at: mockMethod(undefined),
      });

      const result = await getMessageProcessedAt(
        createMockSdk(),
        'ct_mailbox',
        '0xdef',
      );
      expect(result).to.be.null;
    });
  });

  describe('getRecipientIsm', () => {
    it('returns the ISM address for a recipient', async () => {
      mockContractInitialize({
        recipient_ism_for: mockMethod('ct_ismAddr'),
      });

      const result = await getRecipientIsm(
        createMockSdk(),
        'ct_mailbox',
        'ak_recipient',
      );
      expect(result).to.equal('ct_ismAddr');
    });
  });
});
