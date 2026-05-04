import { expect } from 'chai';

import {
  getAnnouncedValidators,
  getAnnouncedStorageLocations,
} from './validator-announce-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('ValidatorAnnounce query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getAnnouncedValidators', () => {
    it('returns list of announced validators', async () => {
      const validators = [
        '0x1234567890abcdef1234567890abcdef12345678',
        '0xabcdef1234567890abcdef1234567890abcdef12',
      ];

      mockContractInitialize({
        get_announced_validators: mockMethod(validators),
      });

      const result = await getAnnouncedValidators(
        createMockSdk(),
        'ct_va',
      );

      expect(result).to.deep.equal(validators);
      expect(result).to.have.length(2);
    });

    it('returns empty list when no validators announced', async () => {
      mockContractInitialize({
        get_announced_validators: mockMethod([]),
      });

      const result = await getAnnouncedValidators(
        createMockSdk(),
        'ct_va',
      );

      expect(result).to.be.empty;
    });
  });

  describe('getAnnouncedStorageLocations', () => {
    it('returns storage locations for each validator', async () => {
      const locations = [
        ['s3://bucket/validator1/'],
        ['s3://bucket/validator2/', 'file:///local/validator2/'],
      ];

      mockContractInitialize({
        get_announced_storage_locations: mockMethod(locations),
      });

      const result = await getAnnouncedStorageLocations(
        createMockSdk(),
        'ct_va',
        ['0xval1', '0xval2'],
      );

      expect(result).to.deep.equal(locations);
      expect(result[0]).to.have.length(1);
      expect(result[1]).to.have.length(2);
    });

    it('returns empty arrays for validators with no locations', async () => {
      mockContractInitialize({
        get_announced_storage_locations: mockMethod([[], []]),
      });

      const result = await getAnnouncedStorageLocations(
        createMockSdk(),
        'ct_va',
        ['0xval1', '0xval2'],
      );

      expect(result[0]).to.be.empty;
      expect(result[1]).to.be.empty;
    });

    it('handles single validator', async () => {
      mockContractInitialize({
        get_announced_storage_locations: mockMethod([
          ['s3://my-bucket/my-validator/'],
        ]),
      });

      const result = await getAnnouncedStorageLocations(
        createMockSdk(),
        'ct_va',
        ['0xsingleval'],
      );

      expect(result).to.have.length(1);
      expect(result[0][0]).to.equal('s3://my-bucket/my-validator/');
    });
  });
});
