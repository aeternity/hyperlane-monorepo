import { expect } from 'chai';

import {
  getTimelockGovernanceConfig,
  getTimelockOperation,
  isTimelockOperationReady,
  getMultiSigConfig,
  getMultiSigTransaction,
} from './governance-query.js';
import {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
} from '../testing/mock-contract.js';

describe('Governance query functions', () => {
  afterEach(() => {
    restoreContractInitialize();
  });

  describe('getTimelockGovernanceConfig', () => {
    it('returns admin and minDelay', async () => {
      mockContractInitialize({
        get_admin: mockMethod('ak_adminAddr'),
        get_min_delay: mockMethod(BigInt(3600)),
      });

      const config = await getTimelockGovernanceConfig(
        createMockSdk(),
        'ct_timelockAddr',
      );

      expect(config.admin).to.equal('ak_adminAddr');
      expect(config.minDelay).to.equal(BigInt(3600));
    });
  });

  describe('getTimelockOperation', () => {
    it('returns operation details when found', async () => {
      mockContractInitialize({
        get_operation: mockMethod({
          target: 'ct_target',
          value: BigInt(100),
          ready_at: BigInt(9999),
          status: 'pending',
        }),
      });

      const op = await getTimelockOperation(
        createMockSdk(),
        'ct_timelockAddr',
        'op_123',
      );

      expect(op).to.not.be.null;
      expect(op!.target).to.equal('ct_target');
      expect(op!.value).to.equal(BigInt(100));
      expect(op!.readyAt).to.equal(BigInt(9999));
      expect(op!.status).to.equal('pending');
    });

    it('returns null when operation not found', async () => {
      mockContractInitialize({
        get_operation: mockMethod(null),
      });

      const op = await getTimelockOperation(
        createMockSdk(),
        'ct_timelockAddr',
        'op_missing',
      );

      expect(op).to.be.null;
    });
  });

  describe('isTimelockOperationReady', () => {
    it('returns true when operation is ready', async () => {
      mockContractInitialize({
        is_ready: mockMethod(true),
      });

      const result = await isTimelockOperationReady(
        createMockSdk(),
        'ct_timelockAddr',
        'op_123',
      );

      expect(result).to.be.true;
    });

    it('returns false when operation is not ready', async () => {
      mockContractInitialize({
        is_ready: mockMethod(false),
      });

      const result = await isTimelockOperationReady(
        createMockSdk(),
        'ct_timelockAddr',
        'op_123',
      );

      expect(result).to.be.false;
    });
  });

  describe('getMultiSigConfig', () => {
    it('returns owners and threshold', async () => {
      const owners = ['ak_owner1', 'ak_owner2', 'ak_owner3'];

      mockContractInitialize({
        get_owners: mockMethod(owners),
        get_threshold: mockMethod(2),
      });

      const config = await getMultiSigConfig(
        createMockSdk(),
        'ct_multisigAddr',
      );

      expect(config.owners).to.deep.equal(owners);
      expect(config.threshold).to.equal(2);
    });

    it('handles single owner', async () => {
      mockContractInitialize({
        get_owners: mockMethod(['ak_singleOwner']),
        get_threshold: mockMethod(1),
      });

      const config = await getMultiSigConfig(
        createMockSdk(),
        'ct_multisigAddr',
      );

      expect(config.owners).to.have.length(1);
      expect(config.threshold).to.equal(1);
    });
  });

  describe('getMultiSigTransaction', () => {
    it('returns transaction details when found', async () => {
      mockContractInitialize({
        get_transaction: mockMethod({
          target: 'ct_target',
          value: BigInt(500),
          executed: false,
          confirm_count: 1,
        }),
      });

      const tx = await getMultiSigTransaction(
        createMockSdk(),
        'ct_multisigAddr',
        0,
      );

      expect(tx).to.not.be.null;
      expect(tx!.target).to.equal('ct_target');
      expect(tx!.value).to.equal(BigInt(500));
      expect(tx!.executed).to.be.false;
      expect(tx!.confirmCount).to.equal(1);
    });

    it('returns null when transaction not found', async () => {
      mockContractInitialize({
        get_transaction: mockMethod(null),
      });

      const tx = await getMultiSigTransaction(
        createMockSdk(),
        'ct_multisigAddr',
        999,
      );

      expect(tx).to.be.null;
    });
  });
});
