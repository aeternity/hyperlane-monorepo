import { expect } from 'chai';

import { AeternitySigner } from './signer.js';
import { AeternityProvider } from './provider.js';
import type { AeternityTransaction } from '../utils/types.js';

describe('AeternitySigner', () => {
  describe('class hierarchy', () => {
    it('extends AeternityProvider', () => {
      expect(AeternitySigner.prototype).to.be.instanceOf(AeternityProvider);
    });
  });

  describe('supportsTransactionBatching', () => {
    it('returns false', async () => {
      const signer = Object.create(AeternitySigner.prototype);
      expect(signer.supportsTransactionBatching()).to.be.false;
    });
  });

  describe('transactionToPrintableJson', () => {
    it('returns a human-readable representation', async () => {
      const signer = Object.create(AeternitySigner.prototype);

      const tx: AeternityTransaction = {
        contractId: 'ct_test',
        entrypoint: 'transfer',
        args: ['ak_recipient', 1000],
        options: {
          amount: BigInt(500),
          gas: 40000,
        },
      };

      const json = await signer.transactionToPrintableJson(tx);
      expect(json).to.deep.equal({
        contractId: 'ct_test',
        entrypoint: 'transfer',
        args: ['ak_recipient', 1000],
        amount: '500',
        gas: 40000,
      });
    });

    it('defaults amount to "0" when not set', async () => {
      const signer = Object.create(AeternitySigner.prototype);

      const tx: AeternityTransaction = {
        contractId: 'ct_test',
        entrypoint: 'balance',
        args: [],
      };

      const json = await signer.transactionToPrintableJson(tx);
      expect(json).to.have.property('amount', '0');
    });

    it('defaults gas to "auto" when not set', async () => {
      const signer = Object.create(AeternitySigner.prototype);

      const tx: AeternityTransaction = {
        contractId: 'ct_test',
        entrypoint: 'balance',
        args: [],
      };

      const json = await signer.transactionToPrintableJson(tx);
      expect(json).to.have.property('gas', 'auto');
    });
  });

  describe('sendAndConfirmBatchTransactions', () => {
    it('throws because batching is not supported', async () => {
      const signer = Object.create(AeternitySigner.prototype);

      try {
        await signer.sendAndConfirmBatchTransactions([]);
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.include('does not support transaction batching');
      }
    });
  });

  describe('connectWithSigner', () => {
    it('rejects empty rpcUrls', async () => {
      try {
        await AeternitySigner.connectWithSigner([], 'testkey');
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.match(/got no rpcUrls/i);
      }
    });
  });
});
