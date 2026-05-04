import { expect } from 'chai';

import {
  buildScheduleOperationTx,
  buildExecuteOperationTx,
  buildCancelOperationTx,
  buildSetMinDelayTx,
  buildAddProposerTx,
  buildRemoveProposerTx,
  buildAddExecutorTx,
  buildRemoveExecutorTx,
  buildSubmitMultiSigTx,
  buildConfirmMultiSigTx,
  buildRevokeMultiSigTx,
  buildExecuteMultiSigTx,
} from './governance-tx.js';

describe('Governance transaction builders', () => {
  const timelockAddress = 'ct_timelockTestAddr';
  const multisigAddress = 'ct_multisigTestAddr';

  describe('buildScheduleOperationTx', () => {
    it('targets the timelock contract with schedule entrypoint', () => {
      const tx = buildScheduleOperationTx(
        timelockAddress,
        'ct_target',
        'calldata',
        BigInt(100),
        BigInt(3600),
      );
      expect(tx.contractId).to.equal(timelockAddress);
      expect(tx.entrypoint).to.equal('schedule');
    });

    it('passes target, callData, value, and delay as arguments', () => {
      const tx = buildScheduleOperationTx(
        timelockAddress,
        'ct_target',
        'calldata',
        BigInt(100),
        BigInt(3600),
      );
      expect(tx.args).to.deep.equal([
        'ct_target',
        'calldata',
        '100',
        '3600',
      ]);
    });

    it('does not set transaction options', () => {
      const tx = buildScheduleOperationTx(
        timelockAddress,
        'ct_target',
        'calldata',
        BigInt(0),
        BigInt(0),
      );
      expect(tx.options).to.be.undefined;
    });
  });

  describe('buildExecuteOperationTx', () => {
    it('targets the timelock with execute entrypoint', () => {
      const tx = buildExecuteOperationTx(timelockAddress, 'op_123');
      expect(tx.contractId).to.equal(timelockAddress);
      expect(tx.entrypoint).to.equal('execute');
      expect(tx.args).to.deep.equal(['op_123']);
    });
  });

  describe('buildCancelOperationTx', () => {
    it('targets the timelock with cancel entrypoint', () => {
      const tx = buildCancelOperationTx(timelockAddress, 'op_123');
      expect(tx.contractId).to.equal(timelockAddress);
      expect(tx.entrypoint).to.equal('cancel');
      expect(tx.args).to.deep.equal(['op_123']);
    });
  });

  describe('buildSetMinDelayTx', () => {
    it('passes new delay as string argument', () => {
      const tx = buildSetMinDelayTx(timelockAddress, BigInt(7200));
      expect(tx.entrypoint).to.equal('set_min_delay');
      expect(tx.args).to.deep.equal(['7200']);
    });
  });

  describe('buildAddProposerTx', () => {
    it('targets the timelock with add_proposer entrypoint', () => {
      const tx = buildAddProposerTx(timelockAddress, 'ak_proposer');
      expect(tx.entrypoint).to.equal('add_proposer');
      expect(tx.args).to.deep.equal(['ak_proposer']);
    });
  });

  describe('buildRemoveProposerTx', () => {
    it('targets the timelock with remove_proposer entrypoint', () => {
      const tx = buildRemoveProposerTx(timelockAddress, 'ak_proposer');
      expect(tx.entrypoint).to.equal('remove_proposer');
      expect(tx.args).to.deep.equal(['ak_proposer']);
    });
  });

  describe('buildAddExecutorTx', () => {
    it('targets the timelock with add_executor entrypoint', () => {
      const tx = buildAddExecutorTx(timelockAddress, 'ak_executor');
      expect(tx.entrypoint).to.equal('add_executor');
      expect(tx.args).to.deep.equal(['ak_executor']);
    });
  });

  describe('buildRemoveExecutorTx', () => {
    it('targets the timelock with remove_executor entrypoint', () => {
      const tx = buildRemoveExecutorTx(timelockAddress, 'ak_executor');
      expect(tx.entrypoint).to.equal('remove_executor');
      expect(tx.args).to.deep.equal(['ak_executor']);
    });
  });

  describe('buildSubmitMultiSigTx', () => {
    it('targets the multisig with submit entrypoint', () => {
      const tx = buildSubmitMultiSigTx(
        multisigAddress,
        'ct_target',
        BigInt(200),
        'data',
      );
      expect(tx.contractId).to.equal(multisigAddress);
      expect(tx.entrypoint).to.equal('submit');
      expect(tx.args).to.deep.equal(['ct_target', '200', 'data']);
    });
  });

  describe('buildConfirmMultiSigTx', () => {
    it('targets the multisig with confirm entrypoint', () => {
      const tx = buildConfirmMultiSigTx(multisigAddress, 0);
      expect(tx.entrypoint).to.equal('confirm');
      expect(tx.args).to.deep.equal([0]);
    });
  });

  describe('buildRevokeMultiSigTx', () => {
    it('targets the multisig with revoke entrypoint', () => {
      const tx = buildRevokeMultiSigTx(multisigAddress, 1);
      expect(tx.entrypoint).to.equal('revoke');
      expect(tx.args).to.deep.equal([1]);
    });
  });

  describe('buildExecuteMultiSigTx', () => {
    it('targets the multisig with execute entrypoint', () => {
      const tx = buildExecuteMultiSigTx(multisigAddress, 2);
      expect(tx.entrypoint).to.equal('execute');
      expect(tx.args).to.deep.equal([2]);
    });
  });
});
