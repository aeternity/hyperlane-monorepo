import { expect } from 'chai';

import {
  getIsmType,
  getMultisigIsmConfig,
  verifyIsm,
} from './ism-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('ISM query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getIsmType', () => {
    it('returns the ISM module type as a number', async () => {
      mockContractInitialize({
        module_type: mockMethod(5),
      });

      const result = await getIsmType(createMockSdk(), 'ct_testIsm');
      expect(result).to.equal(5);
    });

    it('converts bigint results to number', async () => {
      mockContractInitialize({
        module_type: mockMethod(BigInt(3)),
      });

      const result = await getIsmType(createMockSdk(), 'ct_testIsm');
      expect(result).to.equal(3);
    });
  });

  describe('getMultisigIsmConfig', () => {
    it('returns validators and threshold for a domain', async () => {
      const validators = [
        '0x1234567890abcdef1234567890abcdef12345678',
        '0xabcdef1234567890abcdef1234567890abcdef12',
      ];

      mockContractInitialize({
        get_validators: mockMethod(validators),
        get_threshold: mockMethod(2),
      });

      const config = await getMultisigIsmConfig(
        createMockSdk(),
        'ct_testIsm',
        11155111,
      );

      expect(config.validators).to.deep.equal(validators);
      expect(config.threshold).to.equal(2);
    });

    it('handles single validator with threshold 1', async () => {
      const validators = [
        '0xaabbccddee1234567890aabbccddee1234567890',
      ];

      mockContractInitialize({
        get_validators: mockMethod(validators),
        get_threshold: mockMethod(1),
      });

      const config = await getMultisigIsmConfig(
        createMockSdk(),
        'ct_testIsm',
        457,
      );

      expect(config.validators).to.have.length(1);
      expect(config.threshold).to.equal(1);
    });

    it('handles empty validator set', async () => {
      mockContractInitialize({
        get_validators: mockMethod([]),
        get_threshold: mockMethod(0),
      });

      const config = await getMultisigIsmConfig(
        createMockSdk(),
        'ct_testIsm',
        999,
      );

      expect(config.validators).to.be.empty;
      expect(config.threshold).to.equal(0);
    });
  });

  describe('verifyIsm', () => {
    it('returns true when verification succeeds', async () => {
      mockContractInitialize({
        verify: mockMethod(true),
      });

      const result = await verifyIsm(
        createMockSdk(),
        'ct_testIsm',
        new Uint8Array([1, 2, 3]),
        new Uint8Array([4, 5, 6]),
      );

      expect(result).to.be.true;
    });

    it('returns false when verification fails', async () => {
      mockContractInitialize({
        verify: mockMethod(false),
      });

      const result = await verifyIsm(
        createMockSdk(),
        'ct_testIsm',
        new Uint8Array([0]),
        new Uint8Array([0]),
      );

      expect(result).to.be.false;
    });
  });
});
