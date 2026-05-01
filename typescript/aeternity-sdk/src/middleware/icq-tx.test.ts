import { expect } from 'chai';

import {
  buildRegisterViewTargetTx,
  buildEnrollIcqRemoteRouterTx,
} from './icq-tx.js';

describe('ICQ transaction builders', () => {
  const routerAddress = 'ct_icqRouterTestAddr';

  describe('buildRegisterViewTargetTx', () => {
    it('targets the ICQ router with register_view_target', () => {
      const tx = buildRegisterViewTargetTx(routerAddress, 'ct_target');
      expect(tx.contractId).to.equal(routerAddress);
      expect(tx.entrypoint).to.equal('register_view_target');
      expect(tx.args).to.deep.equal(['ct_target']);
    });
  });

  describe('buildEnrollIcqRemoteRouterTx', () => {
    it('targets the ICQ router with enroll_remote_router', () => {
      const tx = buildEnrollIcqRemoteRouterTx(
        routerAddress,
        457,
        '0xremoteRouter',
      );
      expect(tx.entrypoint).to.equal('enroll_remote_router');
      expect(tx.args).to.deep.equal([457, '0xremoteRouter']);
    });
  });
});
