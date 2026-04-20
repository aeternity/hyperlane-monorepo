import { AeSdk, Node, Contract } from '@aeternity/aepp-sdk';

const aciCache = new Map<string, any>();

export async function callStatic(
  nodeUrl: string,
  contractAddress: string,
  aci: any,
  entrypoint: string,
  args: any[] = [],
): Promise<any> {
  const node = new Node(nodeUrl);
  const sdk = new AeSdk({
    nodes: [{ name: 'node', instance: node }],
  });

  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci,
    address: contractAddress as `ct_${string}`,
  });

  const result = await contract[entrypoint](...args);
  return result.decodedResult;
}

export async function initContract(
  sdk: AeSdk,
  aci: any,
  contractAddress: string,
): Promise<any> {
  return Contract.initialize({
    ...sdk.getContext(),
    aci,
    address: contractAddress as `ct_${string}`,
  });
}

export function cacheAci(key: string, aci: any): void {
  aciCache.set(key, aci);
}

export function getCachedAci(key: string): any | undefined {
  return aciCache.get(key);
}
