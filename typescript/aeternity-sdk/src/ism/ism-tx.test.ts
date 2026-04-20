import { expect } from 'chai';

import { buildSetValidatorsAndThresholdTx } from './ism-tx.js';

describe('ISM transaction builders', () => {
  const ismAddress = 'ct_ismTestAddr123';

  describe('buildSetValidatorsAndThresholdTx', () => {
    it('builds a transaction targeting the ISM contract', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, 1, [], 1);
      expect(tx.contractId).to.equal(ismAddress);
    });

    it('uses the set_validators_and_threshold entrypoint', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, 1, [], 1);
      expect(tx.entrypoint).to.equal('set_validators_and_threshold');
    });

    it('passes domain, validators, and threshold as arguments', () => {
      const validators = ['0xaabb', '0xccdd'];
      const tx = buildSetValidatorsAndThresholdTx(
        ismAddress,
        11155111,
        validators,
        2,
      );
      expect(tx.args).to.deep.equal([11155111, validators, 2]);
    });

    it('supports empty validator lists', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, 457, [], 0);
      expect(tx.args[0]).to.equal(457);
      expect(tx.args[1]).to.deep.equal([]);
      expect(tx.args[2]).to.equal(0);
    });

    it('does not set transaction options', () => {
      const tx = buildSetValidatorsAndThresholdTx(ismAddress, 1, [], 1);
      expect(tx.options).to.be.undefined;
    });
  });
});
