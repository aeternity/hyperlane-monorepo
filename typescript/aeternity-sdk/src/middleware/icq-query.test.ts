import { expect } from 'chai';

import { getIcqRouterConfig } from './icq-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('ICQ query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getIcqRouterConfig', () => {
    it('returns owner and deployed block', async () => {
      mockContractInitialize({
        owner: mockMethod('ak_icqOwner'),
        deployed_block: mockMethod(300),
      });

      const config = await getIcqRouterConfig(
        createMockSdk(),
        'ct_icqRouter',
      );

      expect(config.owner).to.equal('ak_icqOwner');
      expect(config.deployedBlock).to.equal(300);
    });
  });
});
