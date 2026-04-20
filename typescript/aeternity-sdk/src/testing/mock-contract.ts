import { Contract } from '@aeternity/aepp-sdk';

export type MockContractMethods = Record<
  string,
  (...args: any[]) => Promise<{ decodedResult: any }>
>;

let originalInitialize: typeof Contract.initialize | undefined;

export function mockMethod(returnValue: any) {
  return async (..._args: any[]) => ({ decodedResult: returnValue });
}

export function mockContractInitialize(methods: MockContractMethods): void {
  if (!originalInitialize) {
    originalInitialize = Contract.initialize;
  }
  (Contract as any).initialize = async (_opts: any) => methods;
}

export function restoreContractInitialize(): void {
  if (originalInitialize) {
    (Contract as any).initialize = originalInitialize;
    originalInitialize = undefined;
  }
}

export function createMockSdk(): any {
  return {
    getContext: () => ({
      onNode: {},
      onCompiler: undefined,
      onAccount: undefined,
    }),
  };
}
