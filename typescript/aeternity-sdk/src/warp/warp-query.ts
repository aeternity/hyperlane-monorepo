import { AeSdk, Contract } from '@aeternity/aepp-sdk';

import { WARP_ROUTER_ACI, AEX9_ACI } from '../aci/index.js';

export async function getWarpRouterConfig(
  sdk: AeSdk,
  routerAddress: string,
): Promise<{
  remoteRouters: Map<number, string>;
  decimalScaling: { numerator: number; denominator: number };
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [WARP_ROUTER_ACI],
    address: routerAddress as `ct_${string}`,
  });

  const [scalingResult, domainsResult] = await Promise.all([
    contract.get_decimal_scaling(),
    contract.get_enrolled_domains(),
  ]);

  const scaling = scalingResult.decodedResult;
  const domains: number[] = domainsResult.decodedResult.map(Number);

  const remoteRouters = new Map<number, string>();
  for (const domain of domains) {
    try {
      const routerResult = await contract.get_remote_router(domain);
      if (routerResult.decodedResult) {
        const routerBytes = routerResult.decodedResult;
        remoteRouters.set(
          domain,
          typeof routerBytes === 'string'
            ? routerBytes
            : Buffer.from(routerBytes).toString('hex'),
        );
      }
    } catch {
      // Domain enrolled but router query failed — skip
    }
  }

  return {
    remoteRouters,
    decimalScaling: {
      numerator: Number(scaling[0]),
      denominator: Number(scaling[1]),
    },
  };
}

export async function getLocalDomain(
  sdk: AeSdk,
  routerAddress: string,
): Promise<number> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [WARP_ROUTER_ACI],
    address: routerAddress as `ct_${string}`,
  });
  const result = await contract.get_local_domain();
  return Number(result.decodedResult);
}

export async function quoteWarpGasPayment(
  sdk: AeSdk,
  routerAddress: string,
  destination: number,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [WARP_ROUTER_ACI],
    address: routerAddress as `ct_${string}`,
  });
  const result = await contract.quote_gas_payment(destination);
  return BigInt(result.decodedResult);
}

export async function verifySetup(
  sdk: AeSdk,
  routerAddress: string,
): Promise<boolean> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [WARP_ROUTER_ACI],
    address: routerAddress as `ct_${string}`,
  });
  const result = await contract.verify_setup();
  return result.decodedResult;
}

export async function quoteTransferRemote(
  sdk: AeSdk,
  routerAddress: string,
  destination: number,
  recipient: string,
  amount: bigint,
): Promise<bigint> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [WARP_ROUTER_ACI],
    address: routerAddress as `ct_${string}`,
  });

  const result = await contract.quote_transfer_remote(
    destination,
    recipient,
    amount,
  );
  return BigInt(result.decodedResult);
}

export async function getAex9TokenMetadata(
  sdk: AeSdk,
  tokenAddress: string,
): Promise<{
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: bigint;
}> {
  const contract = await Contract.initialize({
    ...sdk.getContext(),
    aci: [AEX9_ACI],
    address: tokenAddress as `ct_${string}`,
  });

  const [nameResult, symbolResult, decimalsResult, supplyResult] =
    await Promise.all([
      contract.name(),
      contract.symbol(),
      contract.decimals(),
      contract.total_supply(),
    ]);

  return {
    name: nameResult.decodedResult,
    symbol: symbolResult.decodedResult,
    decimals: Number(decimalsResult.decodedResult),
    totalSupply: BigInt(supplyResult.decodedResult),
  };
}
