import { expect } from 'chai';

import {
  addressToBytes,
  addressToBytes32,
  addressToBytesAeternity,
  bytesToAddressAeternity,
  bytesToProtocolAddress,
  eqAddressAeternity,
  getAddressProtocolType,
  isAddressAeternity,
  isAddressStarknet,
  isValidAddressAeternity,
  isValidAddressStarknet,
  isValidTransactionHash,
  isValidTransactionHashAeternity,
  isZeroishAddress,
  normalizeAddressAeternity,
  padBytesToLength,
} from './addresses.js';
import { ProtocolType } from './types.js';

const ETH_ZERO_ADDR = '0x0000000000000000000000000000000000000000';
const ETH_NON_ZERO_ADDR = '0x0000000000000000000000000000000000000001';
const COS_ZERO_ADDR = 'cosmos1000';
const COS_NON_ZERO_ADDR =
  'neutron1jyyjd3x0jhgswgm6nnctxvzla8ypx50tew3ayxxwkrjfxhvje6kqzvzudq';
const COSMOS_PREFIX = 'neutron';
const COSMOS_NATIVE_ZERO_ADDR =
  '0x0000000000000000000000000000000000000000000000000000000000000000';
const COSMOS_NATIVE_NON_ZERO_ADDR =
  '0x726f757465725f61707000000000000000000000000000010000000000000000';
const SOL_ZERO_ADDR = '111111';
const SOL_NON_ZERO_ADDR = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const STARKNET_ZERO_ADDR =
  '0x0000000000000000000000000000000000000000000000000000000000000000';
const STARKNET_NON_ZERO_ADDR =
  '0x0000000000000000000000000000000000000000000000000000000000000001';
const AE_ACCOUNT_ADDR =
  'ak_2kGmMsDtK1pAo26oAcvqTmvS1gbrxQhBac8ciFu9hP69ssVVMG';
const AE_CONTRACT_ADDR =
  'ct_c3FjvPB8kjngrXDi7Ffkqv6dvhdjawBgLZZANcgQJopkXXGqW';
const AE_TX_HASH =
  'th_2mCk6mAecEhpMG5XRePYVnxMVS33qfpSzSaCRU4MKbBNP8gFjp';

const STARKNET_ADDRESSES = [
  // 65 characters (0x + 63 hex chars)
  '0x5ab3ac43afd012da5037f72691f9791a9fd610900c0a1d6c18d41367aee9a53',
  // 66 characters (0x + 64 hex chars)
  '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7',
  // 63 characters (no 0x prefix)
  '5ab3ac43afd012da5037f72691f9791a9fd610900c0a1d6c18d41367aee9a53',
  // 64 characters (no 0x prefix)
  '049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7',
];

// TODO increase address utility test coverage
describe('Address utilities', () => {
  describe('isZeroishAddress', () => {
    it('Identifies 0-ish addresses', () => {
      expect(isZeroishAddress('0x')).to.be.true;
      expect(isZeroishAddress(ETH_ZERO_ADDR)).to.be.true;
      expect(isZeroishAddress(COS_ZERO_ADDR)).to.be.true;
      expect(isZeroishAddress(COSMOS_NATIVE_ZERO_ADDR)).to.be.true;
      expect(isZeroishAddress(SOL_ZERO_ADDR)).to.be.true;
      expect(isZeroishAddress(STARKNET_ZERO_ADDR)).to.be.true;
    });
    it('Identifies non-0-ish addresses', () => {
      expect(isZeroishAddress(ETH_NON_ZERO_ADDR)).to.be.false;
      expect(isZeroishAddress(COS_NON_ZERO_ADDR)).to.be.false;
      expect(isZeroishAddress(COSMOS_NATIVE_NON_ZERO_ADDR)).to.be.false;
      expect(isZeroishAddress(SOL_NON_ZERO_ADDR)).to.be.false;
      expect(isZeroishAddress(STARKNET_NON_ZERO_ADDR)).to.be.false;
    });
  });

  describe('addressToBytes', () => {
    it('Converts addresses to bytes', () => {
      expect(addressToBytes(ETH_NON_ZERO_ADDR).length).to.equal(32);
      expect(addressToBytes(STARKNET_NON_ZERO_ADDR).length).to.equal(32);
      expect(addressToBytes(AE_ACCOUNT_ADDR).length).to.equal(32);
    });
    it('Rejects zeroish addresses', () => {
      expect(() => addressToBytes(ETH_ZERO_ADDR)).to.throw(Error);
      expect(() => addressToBytes(COS_ZERO_ADDR)).to.throw(Error);
      expect(() => addressToBytes(COSMOS_NATIVE_ZERO_ADDR)).to.throw(Error);
      expect(() => addressToBytes(SOL_ZERO_ADDR)).to.throw(Error);
      expect(() => addressToBytes(STARKNET_ZERO_ADDR)).to.throw(Error);
    });
  });

  describe('padBytesToLength', () => {
    it('Pads bytes to a given length', () => {
      const bytes = new Uint8Array([1, 2, 3]);
      expect(Array.from(padBytesToLength(bytes, 5))).to.deep.equal([
        0, 0, 1, 2, 3,
      ]);
    });
    it('Rejects bytes that exceed the target length', () => {
      const bytes = new Uint8Array([1, 2, 3]);
      expect(() => padBytesToLength(bytes, 2)).to.throw(Error);
    });
  });

  describe('bytesToProtocolAddress', () => {
    it('Converts bytes to address', () => {
      expect(
        bytesToProtocolAddress(
          addressToBytes(ETH_NON_ZERO_ADDR),
          ProtocolType.Ethereum,
        ),
      ).to.equal(ETH_NON_ZERO_ADDR);
      expect(
        bytesToProtocolAddress(
          addressToBytes(COSMOS_NATIVE_NON_ZERO_ADDR),
          ProtocolType.CosmosNative,
          COSMOS_PREFIX,
        ),
      ).to.equal(COSMOS_NATIVE_NON_ZERO_ADDR);
      expect(
        bytesToProtocolAddress(
          addressToBytes(STARKNET_NON_ZERO_ADDR),
          ProtocolType.Starknet,
        ),
      ).to.equal(STARKNET_NON_ZERO_ADDR);
    });
    it('Rejects zeroish addresses', () => {
      expect(() =>
        bytesToProtocolAddress(
          new Uint8Array([0, 0, 0]),
          ProtocolType.Ethereum,
        ),
      ).to.throw(Error);
    });
  });

  describe('isAddressStarknet', () => {
    it('Validates correct Starknet addresses', () => {
      for (const address of STARKNET_ADDRESSES) {
        expect(isAddressStarknet(address)).to.be.true;
      }
    });

    it('Rejects EVM addresses', () => {
      const evmAddress = '0x67C6390e8782b0B4F913f4dA99c065238Fb7DB30';
      expect(isAddressStarknet(evmAddress)).to.be.false;
    });

    it('Rejects addresses exceeding felt252 bounds', () => {
      const outOfBoundsAddress =
        '0x5ab3ac43afd012da5037f72691f9791a9fd610900c0a1d6c18d41367aee9a530';
      expect(isAddressStarknet(outOfBoundsAddress)).to.be.false;
    });
  });

  describe('addressToBytes32', () => {
    it('Converts a base58 Solana address to bytes32 hex', () => {
      // mZhPGteS36G7FhMTcRofLQU8ocBNAsGq7u8SKSHfL2X
      const solAddress = 'mZhPGteS36G7FhMTcRofLQU8ocBNAsGq7u8SKSHfL2X';
      const result = addressToBytes32(solAddress);
      expect(result).to.equal(
        '0x0b6a86806a0354c82b8f049eb75d9c97e370a6f0c0cfa15f47909c3fe1c8f794',
      );
    });
    it('Converts an EVM address to bytes32 hex', () => {
      const result = addressToBytes32(ETH_NON_ZERO_ADDR);
      expect(result).to.equal(
        '0x0000000000000000000000000000000000000000000000000000000000000001',
      );
    });
    it('Returns an already-bytes32 hex address unchanged', () => {
      const bytes32 =
        '0x0b6a86806a0354c82b8f049eb75d9c97e370a6f0c0cfa15f47909c3fe1c8f794';
      expect(addressToBytes32(bytes32)).to.equal(bytes32);
    });
  });

  describe('isValidAddressStarknet', () => {
    it('Validates correct Starknet addresses', () => {
      for (const address of STARKNET_ADDRESSES) {
        expect(isValidAddressStarknet(address)).to.be.true;
      }
    });

    it('Rejects EVM addresses', () => {
      const evmAddress = '0x67C6390e8782b0B4F913f4dA99c065238Fb7DB30';
      expect(isValidAddressStarknet(evmAddress)).to.be.false;
    });

    it('Rejects addresses exceeding felt252 bounds', () => {
      const outOfBoundsAddress =
        '0x5ab3ac43afd012da5037f72691f9791a9fd610900c0a1d6c18d41367aee9a530';
      expect(isValidAddressStarknet(outOfBoundsAddress)).to.be.false;
    });
  });

  describe('isAddressAeternity', () => {
    it('Validates ak_ account addresses', () => {
      expect(isAddressAeternity(AE_ACCOUNT_ADDR)).to.be.true;
    });

    it('Validates ct_ contract addresses', () => {
      expect(isAddressAeternity(AE_CONTRACT_ADDR)).to.be.true;
    });

    it('Rejects EVM addresses', () => {
      expect(isAddressAeternity(ETH_NON_ZERO_ADDR)).to.be.false;
    });

    it('Rejects Solana addresses', () => {
      expect(isAddressAeternity(SOL_NON_ZERO_ADDR)).to.be.false;
    });

    it('Rejects addresses with wrong prefix', () => {
      expect(isAddressAeternity('xx_abc123def456')).to.be.false;
    });

    it('Rejects addresses that are too short', () => {
      expect(isAddressAeternity('ak_abc')).to.be.false;
    });

    it('Rejects empty strings', () => {
      expect(isAddressAeternity('')).to.be.false;
    });
  });

  describe('isValidAddressAeternity', () => {
    it('Validates a real ak_ address with correct checksum', () => {
      expect(isValidAddressAeternity(AE_ACCOUNT_ADDR)).to.be.true;
    });

    it('Validates a real ct_ address with correct checksum', () => {
      expect(isValidAddressAeternity(AE_CONTRACT_ADDR)).to.be.true;
    });

    it('Rejects addresses with invalid characters', () => {
      expect(isValidAddressAeternity('ak_0OIl')).to.be.false;
    });

    it('Rejects non-AE addresses', () => {
      expect(isValidAddressAeternity(ETH_NON_ZERO_ADDR)).to.be.false;
    });

    it('Rejects empty string', () => {
      expect(isValidAddressAeternity('')).to.be.false;
    });
  });

  describe('getAddressProtocolType (Aeternity)', () => {
    it('Detects Aeternity protocol for ak_ addresses', () => {
      expect(getAddressProtocolType(AE_ACCOUNT_ADDR)).to.equal(
        ProtocolType.Aeternity,
      );
    });

    it('Detects Aeternity protocol for ct_ addresses', () => {
      expect(getAddressProtocolType(AE_CONTRACT_ADDR)).to.equal(
        ProtocolType.Aeternity,
      );
    });
  });

  describe('normalizeAddressAeternity', () => {
    it('Returns the address as-is', () => {
      expect(normalizeAddressAeternity(AE_ACCOUNT_ADDR)).to.equal(
        AE_ACCOUNT_ADDR,
      );
    });
  });

  describe('eqAddressAeternity', () => {
    it('Returns true for identical addresses', () => {
      expect(eqAddressAeternity(AE_ACCOUNT_ADDR, AE_ACCOUNT_ADDR)).to.be.true;
    });

    it('Returns false for different addresses', () => {
      expect(eqAddressAeternity(AE_ACCOUNT_ADDR, AE_CONTRACT_ADDR)).to.be
        .false;
    });
  });

  describe('isValidTransactionHashAeternity', () => {
    it('Validates correct th_ hashes', () => {
      expect(isValidTransactionHashAeternity(AE_TX_HASH)).to.be.true;
    });

    it('Rejects EVM tx hashes', () => {
      expect(
        isValidTransactionHashAeternity(
          '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
        ),
      ).to.be.false;
    });

    it('Rejects hashes with wrong prefix', () => {
      expect(isValidTransactionHashAeternity('xx_abc123def456789')).to.be.false;
    });
  });

  describe('isValidTransactionHash (Aeternity)', () => {
    it('Validates via protocol dispatch', () => {
      expect(isValidTransactionHash(AE_TX_HASH, ProtocolType.Aeternity)).to.be
        .true;
    });

    it('Rejects invalid hashes via protocol dispatch', () => {
      expect(isValidTransactionHash('invalid', ProtocolType.Aeternity)).to.be
        .false;
    });
  });

  describe('addressToBytesAeternity', () => {
    it('Extracts 32 bytes from an ak_ address', () => {
      const bytes = addressToBytesAeternity(AE_ACCOUNT_ADDR);
      expect(bytes).to.have.length(32);
      expect(bytes).to.be.instanceOf(Uint8Array);
    });

    it('Extracts 32 bytes from a ct_ address', () => {
      const bytes = addressToBytesAeternity(AE_CONTRACT_ADDR);
      expect(bytes).to.have.length(32);
    });
  });

  describe('bytesToAddressAeternity', () => {
    it('Creates a valid ak_ address from 32 bytes', () => {
      const bytes = addressToBytesAeternity(AE_ACCOUNT_ADDR);
      const reconstructed = bytesToAddressAeternity(bytes);
      expect(reconstructed).to.match(/^ak_/);
      expect(isValidAddressAeternity(reconstructed)).to.be.true;
    });

    it('Round-trips an ak_ address', () => {
      const bytes = addressToBytesAeternity(AE_ACCOUNT_ADDR);
      const reconstructed = bytesToAddressAeternity(bytes);
      expect(reconstructed).to.equal(AE_ACCOUNT_ADDR);
    });

    it('Pads short byte arrays', () => {
      const shortBytes = new Uint8Array([1, 2, 3]);
      const result = bytesToAddressAeternity(shortBytes);
      expect(result).to.match(/^ak_/);
      expect(isValidAddressAeternity(result)).to.be.true;
    });

    it('Truncates long byte arrays', () => {
      const longBytes = new Uint8Array(40).fill(0xab);
      const result = bytesToAddressAeternity(longBytes);
      expect(result).to.match(/^ak_/);
      expect(isValidAddressAeternity(result)).to.be.true;
    });
  });

  describe('addressToBytes / bytesToProtocolAddress (Aeternity)', () => {
    it('Round-trips via the generic address utilities', () => {
      const bytes = addressToBytes(AE_ACCOUNT_ADDR, ProtocolType.Aeternity);
      expect(bytes.length).to.be.greaterThan(0);

      const roundTripped = bytesToProtocolAddress(
        bytes,
        ProtocolType.Aeternity,
      );
      expect(roundTripped).to.equal(AE_ACCOUNT_ADDR);
    });
  });
});
