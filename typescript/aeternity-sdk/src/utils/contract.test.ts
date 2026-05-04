import { expect } from 'chai';

import { cacheAci, getCachedAci } from './contract.js';

describe('Contract utilities', () => {
  describe('cacheAci / getCachedAci', () => {
    it('stores and retrieves an ACI by key', () => {
      const testAci = { contract: { name: 'Test', functions: [] } };
      cacheAci('test-contract', testAci);

      const result = getCachedAci('test-contract');
      expect(result).to.deep.equal(testAci);
    });

    it('returns undefined for non-existent keys', () => {
      const result = getCachedAci('nonexistent-key-' + Date.now());
      expect(result).to.be.undefined;
    });

    it('overwrites existing entries', () => {
      const aci1 = { contract: { name: 'V1' } };
      const aci2 = { contract: { name: 'V2' } };

      cacheAci('overwrite-test', aci1);
      expect(getCachedAci('overwrite-test')).to.deep.equal(aci1);

      cacheAci('overwrite-test', aci2);
      expect(getCachedAci('overwrite-test')).to.deep.equal(aci2);
    });

    it('handles multiple independent keys', () => {
      const aciA = { contract: { name: 'A' } };
      const aciB = { contract: { name: 'B' } };

      cacheAci('key-a', aciA);
      cacheAci('key-b', aciB);

      expect(getCachedAci('key-a')).to.deep.equal(aciA);
      expect(getCachedAci('key-b')).to.deep.equal(aciB);
    });
  });
});
