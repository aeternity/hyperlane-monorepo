import { expect } from 'chai';

import { buildSetHookTx, buildSetRequiredHookTx } from './hook-tx.js';

describe('Hook transaction builders', () => {
  const mailboxAddress = 'ct_mailboxTestAddr123';
  const hookAddress = 'ct_hookTestAddr456';

  describe('buildSetHookTx', () => {
    it('builds a transaction targeting the mailbox contract', () => {
      const tx = buildSetHookTx(mailboxAddress, hookAddress);
      expect(tx.contractId).to.equal(mailboxAddress);
    });

    it('uses the set_default_hook entrypoint', () => {
      const tx = buildSetHookTx(mailboxAddress, hookAddress);
      expect(tx.entrypoint).to.equal('set_default_hook');
    });

    it('passes the hook address as the sole argument', () => {
      const tx = buildSetHookTx(mailboxAddress, hookAddress);
      expect(tx.args).to.deep.equal([hookAddress]);
    });

    it('does not set transaction options', () => {
      const tx = buildSetHookTx(mailboxAddress, hookAddress);
      expect(tx.options).to.be.undefined;
    });
  });

  describe('buildSetRequiredHookTx', () => {
    it('builds a transaction targeting the mailbox contract', () => {
      const tx = buildSetRequiredHookTx(mailboxAddress, hookAddress);
      expect(tx.contractId).to.equal(mailboxAddress);
    });

    it('uses the set_required_hook entrypoint', () => {
      const tx = buildSetRequiredHookTx(mailboxAddress, hookAddress);
      expect(tx.entrypoint).to.equal('set_required_hook');
    });

    it('passes the hook address as the sole argument', () => {
      const tx = buildSetRequiredHookTx(mailboxAddress, hookAddress);
      expect(tx.args).to.deep.equal([hookAddress]);
    });
  });
});
