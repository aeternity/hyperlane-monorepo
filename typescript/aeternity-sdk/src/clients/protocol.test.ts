import { expect } from 'chai';

import { AeternityProtocolProvider } from './protocol.js';

describe('AeternityProtocolProvider', () => {
  let provider: AeternityProtocolProvider;

  beforeEach(() => {
    provider = new AeternityProtocolProvider();
  });

  describe('getMinGas', () => {
    it('returns gas values for all required actions', () => {
      const gas = provider.getMinGas();
      expect(gas).to.have.property('CORE_DEPLOY_GAS');
      expect(gas).to.have.property('WARP_DEPLOY_GAS');
      expect(gas).to.have.property('ISM_DEPLOY_GAS');
      expect(gas).to.have.property('TEST_SEND_GAS');
      expect(gas).to.have.property('AVS_GAS');
    });

    it('returns bigint values', () => {
      const gas = provider.getMinGas();
      expect(typeof gas.CORE_DEPLOY_GAS).to.equal('bigint');
      expect(typeof gas.WARP_DEPLOY_GAS).to.equal('bigint');
      expect(typeof gas.ISM_DEPLOY_GAS).to.equal('bigint');
      expect(typeof gas.TEST_SEND_GAS).to.equal('bigint');
      expect(typeof gas.AVS_GAS).to.equal('bigint');
    });

    it('returns positive gas values', () => {
      const gas = provider.getMinGas();
      expect(gas.CORE_DEPLOY_GAS > 0n).to.be.true;
      expect(gas.WARP_DEPLOY_GAS > 0n).to.be.true;
      expect(gas.ISM_DEPLOY_GAS > 0n).to.be.true;
      expect(gas.TEST_SEND_GAS > 0n).to.be.true;
      expect(gas.AVS_GAS > 0n).to.be.true;
    });
  });

  describe('artifact manager stubs', () => {
    it('createSubmitter throws Not implemented', () => {
      expect(() =>
        provider.createSubmitter({} as any, {} as any),
      ).to.throw('Not implemented');
    });

    it('createIsmArtifactManager throws Not implemented', () => {
      expect(() =>
        provider.createIsmArtifactManager({} as any),
      ).to.throw('Not implemented');
    });

    it('createHookArtifactManager throws Not implemented', () => {
      expect(() =>
        provider.createHookArtifactManager({} as any),
      ).to.throw('Not implemented');
    });

    it('createWarpArtifactManager throws Not implemented', () => {
      expect(() =>
        provider.createWarpArtifactManager({} as any),
      ).to.throw('Not implemented');
    });

    it('createMailboxArtifactManager throws Not implemented', () => {
      expect(() =>
        provider.createMailboxArtifactManager({} as any),
      ).to.throw('Not implemented');
    });

    it('createValidatorAnnounceArtifactManager throws Not implemented', () => {
      expect(() =>
        provider.createValidatorAnnounceArtifactManager({} as any),
      ).to.throw('Not implemented');
    });
  });

  describe('createProvider', () => {
    it('rejects chain metadata without rpcUrls', async () => {
      try {
        await provider.createProvider({} as any);
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.match(/rpc urls undefined/i);
      }
    });
  });

  describe('createSigner', () => {
    it('rejects chain metadata without rpcUrls', async () => {
      try {
        await provider.createSigner({} as any, { privateKey: 'test' } as any);
        expect.fail('should have thrown');
      } catch (e: any) {
        expect(e.message).to.match(/rpc urls undefined/i);
      }
    });
  });
});
