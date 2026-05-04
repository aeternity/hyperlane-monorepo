import { expect } from 'chai';

import {
  buildSetValidatorsAndThresholdTx,
  buildRenounceIsmOwnershipTx,
  buildSetPauseTx,
  buildSetDomainIsmTx,
} from './ism-tx.js';

describe('ISM transaction builders', () => {
  const ismAddress = 'ct_ismTestAddr123';

  describe('buildSetValidatorsAndThresholdTx', () => {
    it('builds a transaction targeting the ISM contract', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, [], 1);
      expect(tx.contractId).to.equal(ismAddress);
    });

    it('uses the set_validators_and_threshold entrypoint', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, [], 1);
      expect(tx.entrypoint).to.equal('set_validators_and_threshold');
    });

    it('passes validators and threshold as arguments', () => {
      const validators = ['0xaabb', '0xccdd'];
      const tx = buildSetValidatorsAndThresholdTx(
        ismAddress,
        validators,
        2,
      );
      expect(tx.args).to.deep.equal([validators, 2]);
    });

    it('supports empty validator lists', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, [], 0);
      expect(tx.args[0]).to.deep.equal([]);
      expect(tx.args[1]).to.equal(0);
    });

    it('does not set transaction options', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, [], 1);
      expect(tx.options).to.be.undefined;
    });
  });

  describe('buildRenounceIsmOwnershipTx', () => {
    it('targets the ISM contract with renounce_ownership', () => {
      const tx = buildRenounceIsmOwnershipTx(ismAddress);
      expect(tx.contractId).to.equal(ismAddress);
      expect(tx.entrypoint).to.equal('renounce_ownership');
      expect(tx.args).to.deep.equal([]);
    });
  });

  describe('buildSetPauseTx', () => {
    it('uses pause entrypoint when paused is true', () => {
      const tx = buildSetPauseTx(ismAddress, true);
      expect(tx.entrypoint).to.equal('pause');
    });

    it('uses unpause entrypoint when paused is false', () => {
      const tx = buildSetPauseTx(ismAddress, false);
      expect(tx.entrypoint).to.equal('unpause');
    });

    it('passes no arguments', () => {
      const tx = buildSetPauseTx(ismAddress, true);
      expect(tx.args).to.deep.equal([]);
    });
  });

  describe('buildSetDomainIsmTx', () => {
    it('targets the routing ISM with set_ism entrypoint', () => {
      const routingIsm = 'ct_routingIsm';
      const targetIsm = 'ct_targetIsm';
      const tx = buildSetDomainIsmTx(routingIsm, 457, targetIsm);
      expect(tx.contractId).to.equal(routingIsm);
      expect(tx.entrypoint).to.equal('set_ism');
      expect(tx.args).to.deep.equal([457, targetIsm]);
    });
  });
});
