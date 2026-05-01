import { expect } from 'chai';

import { getIcaRouterConfig, getLocalIca } from './ica-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('ICA query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getIcaRouterConfig', () => {
    it('returns owner and deployed block', async () => {
      mockContractInitialize({
        owner: mockMethod('ak_routerOwner'),
        deployed_block: mockMethod(200),
      });

      const config = await getIcaRouterConfig(
        createMockSdk(),
        'ct_icaRouter',
      );

      expect(config.owner).to.equal('ak_routerOwner');
      expect(config.deployedBlock).to.equal(200);
    });
  });

  describe('getLocalIca', () => {
    it('returns ICA address when found', async () => {
      mockContractInitialize({
        get_local_ica: mockMethod('ct_localIca'),
      });

      const result = await getLocalIca(
        createMockSdk(),
        'ct_icaRouter',
        457,
        'ak_owner',
      );

      expect(result).to.equal('ct_localIca');
    });

    it('returns null when no ICA found', async () => {
      mockContractInitialize({
        get_local_ica: mockMethod(undefined),
      });

      const result = await getLocalIca(
        createMockSdk(),
        'ct_icaRouter',
        457,
        'ak_unknown',
      );

      expect(result).to.be.null;
    });
  });
});
