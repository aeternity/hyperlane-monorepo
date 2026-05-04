export {
  AE_TESTNET_RPC,
  AE_TESTNET_NETWORK_ID,
  AE_TESTNET_DOMAIN_ID,
  AE_TESTNET_CONTRACTS,
  AE_TESTNET_SEPOLIA_DOMAIN_ID,
} from './constants.js';

export {
  createMockSdk,
  mockContractInitialize,
  restoreContractInitialize,
  mockMethod,
  type MockContractMethods,
} from './mock-contract.js';
