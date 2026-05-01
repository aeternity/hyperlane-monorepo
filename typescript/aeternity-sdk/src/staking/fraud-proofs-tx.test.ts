import { expect } from 'chai';

import {
  buildAttributePrematureTx,
  buildAttributeMessageIdTx,
  buildAttributeRootTx,
  buildWhitelistMerkleTreeTx,
} from './fraud-proofs-tx.js';

describe('Fraud proofs transaction builders', () => {
  const address = 'ct_fraudProofsTestAddr';
  const checkpoint = { root: '0xroot', index: 5 };

  describe('buildAttributePrematureTx', () => {
    it('targets the fraud proofs contract with attribute_premature', () => {
      const tx = buildAttributePrematureTx(address, checkpoint, '0xsig');
      expect(tx.contractId).to.equal(address);
      expect(tx.entrypoint).to.equal('attribute_premature');
      expect(tx.args).to.deep.equal([checkpoint, '0xsig']);
    });
  });

  describe('buildAttributeMessageIdTx', () => {
    it('passes checkpoint, proof, message ID, and signature', () => {
      const tx = buildAttributeMessageIdTx(
        address,
        checkpoint,
        ['0xp1', '0xp2'],
        '0xmsgId',
        '0xsig',
      );
      expect(tx.entrypoint).to.equal('attribute_message_id');
      expect(tx.args).to.deep.equal([
        checkpoint,
        ['0xp1', '0xp2'],
        '0xmsgId',
        '0xsig',
      ]);
    });
  });

  describe('buildAttributeRootTx', () => {
    it('passes checkpoint, proof, and signature', () => {
      const tx = buildAttributeRootTx(
        address,
        checkpoint,
        ['0xp1'],
        '0xsig',
      );
      expect(tx.entrypoint).to.equal('attribute_root');
      expect(tx.args).to.deep.equal([checkpoint, ['0xp1'], '0xsig']);
    });
  });

  describe('buildWhitelistMerkleTreeTx', () => {
    it('targets the contract with whitelist_merkle_tree', () => {
      const tx = buildWhitelistMerkleTreeTx(address, 'ct_tree');
      expect(tx.entrypoint).to.equal('whitelist_merkle_tree');
      expect(tx.args).to.deep.equal(['ct_tree']);
    });
  });
});
