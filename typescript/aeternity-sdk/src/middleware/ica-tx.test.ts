import { expect } from 'chai';

import {
  buildCallRemoteAeTx,
  buildEnrollIcaRemoteRouterTx,
} from './ica-tx.js';

describe('ICA transaction builders', () => {
  const routerAddress = 'ct_icaRouterTestAddr';

  describe('buildCallRemoteAeTx', () => {
    it('targets the ICA router with call_remote_ae entrypoint', () => {
      const tx = buildCallRemoteAeTx(routerAddress, 457, []);
      expect(tx.contractId).to.equal(routerAddress);
      expect(tx.entrypoint).to.equal('call_remote_ae');
    });

    it('encodes recipients with string amounts', () => {
      const recipients: Array<[string, bigint]> = [
        ['ak_recipient1', BigInt(100)],
        ['ak_recipient2', BigInt(200)],
      ];
      const tx = buildCallRemoteAeTx(routerAddress, 457, recipients);
      expect(tx.args[0]).to.equal(457);
      expect(tx.args[1]).to.deep.equal([
        ['ak_recipient1', '100'],
        ['ak_recipient2', '200'],
      ]);
    });

    it('handles empty recipients list', () => {
      const tx = buildCallRemoteAeTx(routerAddress, 457, []);
      expect(tx.args[1]).to.deep.equal([]);
    });
  });

  describe('buildEnrollIcaRemoteRouterTx', () => {
    it('targets the ICA router with enroll_remote_router', () => {
      const tx = buildEnrollIcaRemoteRouterTx(
        routerAddress,
        457,
        '0xremoteRouter',
      );
      expect(tx.entrypoint).to.equal('enroll_remote_router');
      expect(tx.args).to.deep.equal([457, '0xremoteRouter']);
    });
  });
});
