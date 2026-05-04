import { expect } from 'chai';

import { AeternityProvider } from './provider.js';

describe('AeternityProvider', () => {
  describe('constructor', () => {
    it('stores rpcUrls', () => {
      const urls = ['https://testnet.aeternity.io'];
      const provider = new AeternityProvider(urls, {} as any);
      expect(provider.getRpcUrls()).to.deep.equal(urls);
    });

    it('exposes the AeSdk instance', () => {
      const provider = new AeternityProvider(['https://test'], {} as any);
      const sdk = provider.getSdk();
      expect(sdk).to.not.be.undefined;
    });

    it('exposes the Node instance', () => {
      const mockNode = { getStatus: async () => ({}) };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
      );
      expect(provider.getNode()).to.equal(mockNode);
    });
  });

  describe('connect', () => {
    it('rejects empty rpcUrls', async () => {
      try {
        await AeternityProvider.connect([]);
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.match(/got no rpcUrls/i);
      }
    });
  });

  describe('estimateTransactionFee', () => {
    it('estimates fee from gas and gasPrice', async () => {
      const provider = new AeternityProvider(
        ['https://test'],
        {} as any,
      );

      const result = await provider.estimateTransactionFee({
        transaction: {
          contractId: 'ct_test',
          entrypoint: 'test',
          args: [],
          options: { gas: 20000, gasPrice: 2000000000 },
        },
      });

      expect(result.gasUnits).to.equal(BigInt(20000));
      expect(result.gasPrice).to.equal(2000000000);
      expect(result.fee).to.equal(BigInt(20000) * BigInt(2000000000));
    });

    it('uses default gas when options are absent', async () => {
      const provider = new AeternityProvider(
        ['https://test'],
        {} as any,
      );

      const result = await provider.estimateTransactionFee({
        transaction: {
          contractId: 'ct_test',
          entrypoint: 'test',
          args: [],
        },
      });

      expect(result.gasUnits).to.equal(BigInt(50000));
      expect(result.gasPrice).to.equal(1000000000);
      expect(result.fee).to.equal(BigInt(50000) * BigInt(1000000000));
    });
  });

  describe('isHealthy', () => {
    it('returns true when node is not syncing', async () => {
      const mockNode = {
        getStatus: async () => ({ syncing: false }),
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
      );

      const healthy = await provider.isHealthy();
      expect(healthy).to.be.true;
    });

    it('returns false when node is syncing', async () => {
      const mockNode = {
        getStatus: async () => ({ syncing: true }),
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
      );

      const healthy = await provider.isHealthy();
      expect(healthy).to.be.false;
    });

    it('returns false when node is unreachable', async () => {
      const mockNode = {
        getStatus: async () => {
          throw new Error('Connection refused');
        },
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
      );

      const healthy = await provider.isHealthy();
      expect(healthy).to.be.false;
    });
  });

  describe('getHeight', () => {
    it('returns the current key block height', async () => {
      const mockNode = {
        getCurrentKeyBlockHeight: async () => 42567,
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
      );

      const height = await provider.getHeight();
      expect(height).to.equal(42567);
    });
  });

  describe('getBalance', () => {
    it('returns native balance for ak_ addresses', async () => {
      const mockNode = {
        getAccountByPubkey: async (_addr: string) => ({
          balance: '5000000000000000000',
        }),
      };
      const mockSdk = {
        getContext: () => ({}),
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
        mockSdk as any,
      );

      const balance = await provider.getBalance({
        address: 'ak_testAddr',
        denom: '',
      });
      expect(balance).to.equal(BigInt('5000000000000000000'));
    });

    it('returns 0 when account does not exist', async () => {
      const mockNode = {
        getAccountByPubkey: async () => {
          throw new Error('Account not found');
        },
      };
      const mockSdk = {
        getContext: () => ({}),
      };
      const provider = new AeternityProvider(
        ['https://test'],
        mockNode as any,
        mockSdk as any,
      );

      const balance = await provider.getBalance({
        address: 'ak_nonexistent',
        denom: '',
      });
      expect(balance).to.equal(BigInt(0));
    });
  });

  describe('getTotalSupply', () => {
    it('throws for native AE', async () => {
      const provider = new AeternityProvider(
        ['https://test'],
        {} as any,
      );

      try {
        await provider.getTotalSupply({ denom: '' });
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.include('Native AE has no total supply');
      }
    });
  });
});
