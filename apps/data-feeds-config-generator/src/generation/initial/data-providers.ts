import { selectDirectory } from '@blocksense/base-utils/fs';

import { keysOf } from '@blocksense/base-utils/array-iter';
import {
  Pair,
  CryptoPriceFeedsArgs,
} from '@blocksense/config-types/data-feeds-config';

import { artifactsDir } from '../../paths';
import { AssetInfo } from '../../data-services/exchange-assets';
import { SimplifiedFeed } from './types';

export type ProviderData = {
  name: string;
  type: 'exchanges' | 'aggregators';
  data: AssetInfo[];
};

export async function addDataProviders(
  dataFeeds: SimplifiedFeed[],
  exchangeAssetsMap: ProviderData[],
) {
  // Filter out feeds without a quote pair
  const filteredFeeds = filterFeedsWithQuotes(dataFeeds);

  // Map feeds with providers
  const dataFeedsWithCryptoResources = await Promise.all(
    filteredFeeds.map(async feed => {
      const providers = getAllProvidersForFeed(feed, exchangeAssetsMap);
      return {
        ...feed,
        additional_feed_info: {
          ...feed.additional_feed_info,
          arguments: providers,
        },
      };
    }),
  );

  // Filter out feeds without exchange providers
  const feedsWithExchangeProviders = dataFeedsWithCryptoResources.filter(
    feed => 'exchanges' in feed.additional_feed_info.arguments,
  );

  return feedsWithExchangeProviders;
}

// Function to get all providers for a feed
function getAllProvidersForFeed(
  feed: SimplifiedFeed,
  exchangeAssets: ProviderData[],
): CryptoPriceFeedsArgs {
  const providers: CryptoPriceFeedsArgs = {};

  const addProvider = (key: string, type: ProviderData['type'], value: any) => {
    if (value) {
      providers[type] ??= {};
      providers[type][key] = value;
    }
  };

  exchangeAssets.forEach(exchangeData => {
    addProvider(
      exchangeData.name,
      exchangeData.type,
      getPriceFeedDataProvidersInfo(feed, exchangeData.data),
    );
  });

  return providers;
}

/**
 * Get provider resources for a given feed.
 *
 * This code takes an array of supported assets by an exchange and reduces it into a single object (`providerInfo`).
 * Each key in the `providerInfo` object corresponds to a key found in the `data` property of the assets.
 * The values are arrays that aggregate all the values associated with that key across all assets.
 * If no data is aggregated (i.e., `providerInfo` has no keys), the function returns `null`.
 * Otherwise, it returns the `providerInfo` object.
 */
function getPriceFeedDataProvidersInfo<T>(
  feed: SimplifiedFeed,
  exchangeAssets: AssetInfo[],
  includeStableCoins: boolean = true,
): Record<string, string[]> | null {
  const supportedAssets = exchangeAssets.filter(symbol =>
    isPairSupportedByCryptoProvider(
      feed.additional_feed_info.pair,
      symbol.pair,
      includeStableCoins,
    ),
  );
  const providerInfo = supportedAssets.reduce(
    (acc, asset) => {
      keysOf(asset.data).forEach(key => {
        const curr = acc[key] ?? [];
        acc[key] = [...curr, asset.data[key]].sort();
      });
      return acc;
    },
    {} as Record<string, string[]>,
  );

  if (keysOf(providerInfo).length === 0) {
    return null;
  }

  return providerInfo;
}

// Filter feeds that have a quote
function filterFeedsWithQuotes(feeds: SimplifiedFeed[]): SimplifiedFeed[] {
  return feeds.filter(feed => feed.additional_feed_info.pair.quote);
}

// Save data to JSON file
async function saveToFile(data: any, fileName: string) {
  const { writeJSON } = selectDirectory(artifactsDir);
  await writeJSON({ content: { data }, name: fileName });
}

// Placeholder for now; to be extended later
export const stableCoins = {
  USD: ['USDT', 'USDC'],
  EUR: ['EURC', 'EURS', 'EURt'],
};

function equalsCaseInsensitive(a: string, b: string) {
  // FIXME: find why the following causes the program to hang:
  // return a.localeCompare(b, undefined, { sensitivity: 'base' }) === 0;
  return a.toLowerCase() === b.toLowerCase();
}

// Pair validation logic
function isPairSupportedByCryptoProvider(
  oracleFeedPair: Pair,
  dataProviderPair: Pair,
  includeStableCoins: boolean = true,
): boolean {
  const isBaseCompatible = equalsCaseInsensitive(
    oracleFeedPair.base,
    dataProviderPair.base,
  );
  const isCompatibleQuote =
    equalsCaseInsensitive(oracleFeedPair.quote, dataProviderPair.quote) ||
    (includeStableCoins &&
      oracleFeedPair.quote in stableCoins &&
      // Consider stablecoins quotings equivalent to fiat quotings:
      stableCoins[oracleFeedPair.quote as keyof typeof stableCoins].includes(
        dataProviderPair.quote,
      ));

  return isBaseCompatible && isCompatibleQuote;
}

export function detectOutliers(
  feedPriceData: Record<string, Record<string, number>[]>,
): string[] {
  const outlinerts: string[] = [];
  const [asset, pricesArray] = Object.entries(feedPriceData)[0];

  const zeroPricesFiltered = pricesArray.filter(priceData => {
    const [exchangeName, price] = Object.entries(priceData)[0];
    if (price === 0 || price < 0 || price === null) {
      console.log(
        `Detected Outlier : ${exchangeName} (${price}) for feed ${asset}`,
      );
      outlinerts.push(exchangeName);
      return false;
    }
    return true;
  });

  // Convert array of { Exchange: Price } into a single object { Exchange: Price }
  const prices: Record<string, number> = Object.assign(
    {},
    ...zeroPricesFiltered,
  );
  const exchanges = Object.keys(prices);
  const priceValues = Object.values(prices);

  const sortedPrices = [...priceValues].sort((a, b) => a - b);
  const middle = Math.floor(sortedPrices.length / 2);
  const medianPrice =
    sortedPrices.length % 2 === 0
      ? (sortedPrices[middle - 1] + sortedPrices[middle]) / 2
      : sortedPrices[middle];

  exchanges.forEach(exchange => {
    const price = prices[exchange];
    if (Math.abs(price - medianPrice) / medianPrice > 0.1) {
      console.log(
        `Detected Outlier : ${exchange} (${price}) for feed ${asset}`,
      );
      outlinerts.push(exchange);
    }
  });
  return outlinerts;
}
