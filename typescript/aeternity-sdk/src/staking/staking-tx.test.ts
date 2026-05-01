import { expect } from 'chai';

import {
  buildStakeTx,
  buildInitiateUnstakeTx,
  buildCompleteUnstakeTx,
  buildSetSlasherTx,
  buildSetMinStakeTx,
  buildSlashForFraudTx,
} from './staking-tx.js';

describe('Staking transaction builders', () => {
  const stakingAddress = 'ct_stakingTestAddr';
  const slasherAddress = 'ct_slasherTestAddr';

  describe('buildStakeTx', () => {
    it('targets the staking contract with stake entrypoint', () => {
      const tx = buildStakeTx(stakingAddress, '0xsigkey', BigInt(1000));
      expect(tx.contractId).to.equal(stakingAddress);
      expect(tx.entrypoint).to.equal('stake');
    });

    it('passes signing key as argument', () => {
      const tx = buildStakeTx(stakingAddress, '0xsigkey', BigInt(1000));
      expect(tx.args).to.deep.equal(['0xsigkey']);
    });

    it('sets amount in transaction options', () => {
      const tx = buildStakeTx(stakingAddress, '0xsigkey', BigInt(5000));
      expect(tx.options).to.deep.equal({ amount: BigInt(5000) });
    });
  });

  describe('buildInitiateUnstakeTx', () => {
    it('targets the staking contract with initiate_unstake', () => {
      const tx = buildInitiateUnstakeTx(stakingAddress);
      expect(tx.contractId).to.equal(stakingAddress);
      expect(tx.entrypoint).to.equal('initiate_unstake');
      expect(tx.args).to.deep.equal([]);
    });
  });

  describe('buildCompleteUnstakeTx', () => {
    it('targets the staking contract with complete_unstake', () => {
      const tx = buildCompleteUnstakeTx(stakingAddress);
      expect(tx.contractId).to.equal(stakingAddress);
      expect(tx.entrypoint).to.equal('complete_unstake');
      expect(tx.args).to.deep.equal([]);
    });
  });

  describe('buildSetSlasherTx', () => {
    it('targets the staking contract with set_slasher', () => {
      const tx = buildSetSlasherTx(stakingAddress, 'ct_slasher');
      expect(tx.entrypoint).to.equal('set_slasher');
      expect(tx.args).to.deep.equal(['ct_slasher']);
    });
  });

  describe('buildSetMinStakeTx', () => {
    it('passes min stake as string argument', () => {
      const tx = buildSetMinStakeTx(stakingAddress, BigInt(2000));
      expect(tx.entrypoint).to.equal('set_min_stake');
      expect(tx.args).to.deep.equal(['2000']);
    });
  });

  describe('buildSlashForFraudTx', () => {
    it('targets the slasher contract with slash_for_fraud', () => {
      const tx = buildSlashForFraudTx(slasherAddress, 'ak_val1', 'proof_123');
      expect(tx.contractId).to.equal(slasherAddress);
      expect(tx.entrypoint).to.equal('slash_for_fraud');
      expect(tx.args).to.deep.equal(['ak_val1', 'proof_123']);
    });
  });
});
