import { expect } from 'chai';

import {
  AE_TESTNET_RPC,
  AE_TESTNET_NETWORK_ID,
  AE_TESTNET_DOMAIN_ID,
  AE_TESTNET_CONTRACTS,
  AE_TESTNET_SEPOLIA_DOMAIN_ID,
} from './constants.js';

describe('Testing constants', () => {
  it('defines the testnet RPC URL', () => {
    expect(AE_TESTNET_RPC).to.be.a('string');
    expect(AE_TESTNET_RPC).to.match(/^https?:\/\//);
  });

  it('defines the testnet network ID', () => {
    expect(AE_TESTNET_NETWORK_ID).to.equal('ae_uat');
  });

  it('defines a numeric domain ID', () => {
    expect(AE_TESTNET_DOMAIN_ID).to.be.a('number');
    expect(AE_TESTNET_DOMAIN_ID).to.equal(457);
  });

  it('defines the Sepolia domain ID', () => {
    expect(AE_TESTNET_SEPOLIA_DOMAIN_ID).to.equal(11155111);
  });

  it('defines all required contract addresses', () => {
    expect(AE_TESTNET_CONTRACTS).to.have.property('mailbox');
    expect(AE_TESTNET_CONTRACTS).to.have.property('merkleTreeHook');
    expect(AE_TESTNET_CONTRACTS).to.have.property('multisigIsm');
    expect(AE_TESTNET_CONTRACTS).to.have.property('validatorAnnounce');
    expect(AE_TESTNET_CONTRACTS).to.have.property('noopHook');
    expect(AE_TESTNET_CONTRACTS).to.have.property('hypNativeAE');
    expect(AE_TESTNET_CONTRACTS).to.have.property('wethToken');
    expect(AE_TESTNET_CONTRACTS).to.have.property('hypAEX9Synthetic');
  });

  it('all contract addresses use the ct_ prefix', () => {
    for (const [, address] of Object.entries(AE_TESTNET_CONTRACTS)) {
      expect(address).to.match(/^ct_/);
    }
  });
});
