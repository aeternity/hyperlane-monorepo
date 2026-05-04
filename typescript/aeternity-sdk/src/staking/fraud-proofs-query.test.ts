import { expect } from 'chai';

import {
  isPrematureCheckpoint,
  isFraudulentMessageId,
  isFraudulentRoot,
  getFraudAttribution,
} from './fraud-proofs-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Fraud proofs query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  const checkpoint = { root: '0xroot', index: 5, merkle_tree: 'ct_tree' };

  describe('isPrematureCheckpoint', () => {
    it('returns true for premature checkpoint', async () => {
      mockContractInitialize({
        is_premature: mockMethod(true),
      });

      const result = await isPrematureCheckpoint(
        createMockSdk(),
        'ct_fraudProofs',
        checkpoint,
      );

      expect(result).to.be.true;
    });

    it('returns false for valid checkpoint', async () => {
      mockContractInitialize({
        is_premature: mockMethod(false),
      });

      const result = await isPrematureCheckpoint(
        createMockSdk(),
        'ct_fraudProofs',
        checkpoint,
      );

      expect(result).to.be.false;
    });
  });

  describe('isFraudulentMessageId', () => {
    it('returns true for fraudulent message ID', async () => {
      mockContractInitialize({
        is_fraudulent_message_id: mockMethod(true),
      });

      const result = await isFraudulentMessageId(
        createMockSdk(),
        'ct_fraudProofs',
        checkpoint,
        ['0xproof1'],
        '0xactualId',
      );

      expect(result).to.be.true;
    });

    it('returns false for valid message ID', async () => {
      mockContractInitialize({
        is_fraudulent_message_id: mockMethod(false),
      });

      const result = await isFraudulentMessageId(
        createMockSdk(),
        'ct_fraudProofs',
        checkpoint,
        ['0xproof1'],
        '0xactualId',
      );

      expect(result).to.be.false;
    });
  });

  describe('isFraudulentRoot', () => {
    it('returns true for fraudulent root', async () => {
      mockContractInitialize({
        is_fraudulent_root: mockMethod(true),
      });

      const result = await isFraudulentRoot(
        createMockSdk(),
        'ct_fraudProofs',
        checkpoint,
        ['0xproof1', '0xproof2'],
      );

      expect(result).to.be.true;
    });
  });

  describe('getFraudAttribution', () => {
    it('returns attribution details when found', async () => {
      mockContractInitialize({
        get_attribution: mockMethod({
          signer: 'ak_signer',
          fraud_type: 2,
          timestamp: BigInt(1700000000),
        }),
      });

      const result = await getFraudAttribution(
        createMockSdk(),
        'ct_attributeAddr',
        checkpoint,
        '0xsig',
      );

      expect(result).to.not.be.null;
      expect(result!.signer).to.equal('ak_signer');
      expect(result!.fraudType).to.equal(2);
      expect(result!.timestamp).to.equal(BigInt(1700000000));
    });

    it('returns null when no attribution found', async () => {
      mockContractInitialize({
        get_attribution: mockMethod(null),
      });

      const result = await getFraudAttribution(
        createMockSdk(),
        'ct_attributeAddr',
        checkpoint,
        '0xsig',
      );

      expect(result).to.be.null;
    });
  });
});
