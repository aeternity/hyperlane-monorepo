import { expect } from 'chai';

import {
  getStakingConfig,
  getActiveValidators,
  isActiveValidator,
  getValidatorWeight,
  getFraudSlasherConfig,
} from './staking-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Staking query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getStakingConfig', () => {
    it('returns full staking configuration', async () => {
      mockContractInitialize({
        owner: mockMethod('ak_ownerAddr'),
        min_stake: mockMethod(BigInt(1000)),
        unstake_delay: mockMethod(BigInt(100)),
        get_total_staked: mockMethod(BigInt(50000)),
        deployed_block: mockMethod(42),
      });

      const config = await getStakingConfig(
        createMockSdk(),
        'ct_stakingAddr',
      );

      expect(config.owner).to.equal('ak_ownerAddr');
      expect(config.minStake).to.equal(BigInt(1000));
      expect(config.unstakeDelay).to.equal(BigInt(100));
      expect(config.totalStaked).to.equal(BigInt(50000));
      expect(config.deployedBlock).to.equal(42);
    });
  });

  describe('getActiveValidators', () => {
    it('returns list of active validator addresses', async () => {
      const validators = ['ak_val1', 'ak_val2'];

      mockContractInitialize({
        get_active_validators: mockMethod(validators),
      });

      const result = await getActiveValidators(
        createMockSdk(),
        'ct_stakingAddr',
      );

      expect(result).to.deep.equal(validators);
    });

    it('handles empty validator set', async () => {
      mockContractInitialize({
        get_active_validators: mockMethod([]),
      });

      const result = await getActiveValidators(
        createMockSdk(),
        'ct_stakingAddr',
      );

      expect(result).to.be.empty;
    });
  });

  describe('isActiveValidator', () => {
    it('returns true for active validator', async () => {
      mockContractInitialize({
        is_active_validator: mockMethod(true),
      });

      const result = await isActiveValidator(
        createMockSdk(),
        'ct_stakingAddr',
        'ak_val1',
      );

      expect(result).to.be.true;
    });

    it('returns false for inactive validator', async () => {
      mockContractInitialize({
        is_active_validator: mockMethod(false),
      });

      const result = await isActiveValidator(
        createMockSdk(),
        'ct_stakingAddr',
        'ak_unknown',
      );

      expect(result).to.be.false;
    });
  });

  describe('getValidatorWeight', () => {
    it('returns validator weight as bigint', async () => {
      mockContractInitialize({
        get_validator_weight: mockMethod(BigInt(5000)),
      });

      const result = await getValidatorWeight(
        createMockSdk(),
        'ct_stakingAddr',
        'ak_val1',
      );

      expect(result).to.equal(BigInt(5000));
    });
  });

  describe('getFraudSlasherConfig', () => {
    it('returns deployed block number', async () => {
      mockContractInitialize({
        deployed_block: mockMethod(100),
      });

      const config = await getFraudSlasherConfig(
        createMockSdk(),
        'ct_slasherAddr',
      );

      expect(config.deployedBlock).to.equal(100);
    });
  });
});
