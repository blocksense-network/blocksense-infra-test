// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {HistoricDataFeedStoreGenericV1} from '../../HistoricDataFeedStoreGenericV1.sol';
import {TransmissionUtils} from '../../libraries/TransmissionUtils.sol';

abstract contract HistoricConsumer {
  address public immutable dataFeedStore;
  mapping(uint32 => mapping(uint32 => TransmissionUtils.Data))
    internal dataFeeds;
  mapping(uint32 => uint32) public counters;
  mapping(uint32 => TransmissionUtils.Data) public latestDataFeeds;

  error GetFeedByIdFailed();

  constructor(address _dataFeedStore) {
    dataFeedStore = _dataFeedStore;
  }

  function getFeedById(
    uint32 key
  ) external view returns (TransmissionUtils.Data memory) {
    return dataFeeds[key][counters[key]];
  }

  function getFeedAtCounter(
    uint32 key,
    uint32 counter
  ) external view returns (TransmissionUtils.Data memory) {
    return dataFeeds[key][counter];
  }

  function setMultipleFetchedLatestFeedsById(uint32[] calldata keys) external {
    for (uint i = 0; i < keys.length; i++) {
      uint32 counter = _getLatestCounter(keys[i]);
      dataFeeds[keys[i]][counter] = _getFeedById(keys[i]);
      counters[keys[i]] = counter;
    }
  }

  function setMultipleLatestFeedsById(uint32[] calldata keys) external {
    for (uint i = 0; i < keys.length; i++) {
      latestDataFeeds[keys[i]] = _getFeedById(keys[i]);
    }
  }

  function setMultipleFeedsAtCounter(
    uint32[] calldata keys,
    uint32[] calldata counters_
  ) external {
    for (uint i = 0; i < keys.length; i++) {
      dataFeeds[keys[i]][counters_[i]] = _getFeedAtCounter(
        keys[i],
        counters_[i]
      );
    }
  }

  function _getFeedById(
    uint32 key
  ) internal view virtual returns (TransmissionUtils.Data memory);

  function _getLatestCounter(uint32 key) internal view virtual returns (uint32);

  function _getFeedAtCounter(
    uint32 key,
    uint32 counter
  ) internal view virtual returns (TransmissionUtils.Data memory);
}
