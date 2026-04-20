import { expect } from 'chai';

import {
  getWarpRouterConfig,
  quoteTransferRemote,
  getAex9TokenMetadata,
} from './warp-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Warp query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getWarpRouterConfig', () => {
    it('returns router configuration with decimal scaling', async () => {
      mockContractInitialize({
        get_decimal_scaling: mockMethod([BigInt(1), BigInt(1)]),
      });

      const config = await getWarpRouterConfig(
        createMockSdk(),
        'ct_router',
      );

      expect(config.decimalScaling.numerator).to.equal(1);
      expect(config.decimalScaling.denominator).to.equal(1);
      expect(config.remoteRouters).to.be.instanceOf(Map);
    });

    it('handles non-trivial decimal scaling', async () => {
      mockContractInitialize({
        get_decimal_scaling: mockMethod([BigInt(10), BigInt(18)]),
      });

      const config = await getWarpRouterConfig(
        createMockSdk(),
        'ct_router',
      );

      expect(config.decimalScaling.numerator).to.equal(10);
      expect(config.decimalScaling.denominator).to.equal(18);
    });
  });

  describe('quoteTransferRemote', () => {
    it('returns the transfer fee as bigint', async () => {
      mockContractInitialize({
        quote_transfer_remote: mockMethod(BigInt(1500)),
      });

      const fee = await quoteTransferRemote(
        createMockSdk(),
        'ct_router',
        11155111,
        '0xrecipient',
        BigInt(1000000),
      );

      expect(fee).to.equal(BigInt(1500));
    });

    it('returns zero for fee-free transfers', async () => {
      mockContractInitialize({
        quote_transfer_remote: mockMethod(0),
      });

      const fee = await quoteTransferRemote(
        createMockSdk(),
        'ct_router',
        457,
        '0xrecipient',
        BigInt(500),
      );

      expect(fee).to.equal(BigInt(0));
    });
  });

  describe('getAex9TokenMetadata', () => {
    it('returns full AEX-9 token metadata', async () => {
      mockContractInitialize({
        name: mockMethod('Wrapped Ether'),
        symbol: mockMethod('WETH'),
        decimals: mockMethod(18),
        total_supply: mockMethod(BigInt('1000000000000000000000')),
      });

      const metadata = await getAex9TokenMetadata(
        createMockSdk(),
        'ct_token',
      );

      expect(metadata.name).to.equal('Wrapped Ether');
      expect(metadata.symbol).to.equal('WETH');
      expect(metadata.decimals).to.equal(18);
      expect(metadata.totalSupply).to.equal(
        BigInt('1000000000000000000000'),
      );
    });

    it('handles tokens with zero decimals', async () => {
      mockContractInitialize({
        name: mockMethod('Simple Token'),
        symbol: mockMethod('SMP'),
        decimals: mockMethod(0),
        total_supply: mockMethod(BigInt(1000)),
      });

      const metadata = await getAex9TokenMetadata(
        createMockSdk(),
        'ct_simple',
      );

      expect(metadata.decimals).to.equal(0);
      expect(metadata.totalSupply).to.equal(BigInt(1000));
    });
  });
});
