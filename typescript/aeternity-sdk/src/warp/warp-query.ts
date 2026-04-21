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

  const remoteRouters = new Map<number, string>();

  const scalingResult = await contract.get_decimal_scaling();
  const scaling = scalingResult.decodedResult;

  return {
    remoteRouters,
    decimalScaling: {
      numerator: Number(scaling[0]),
      denominator: Number(scaling[1]),
    },
  };
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
