// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title Blocksense
/// @notice Library for calling dataFeedStore functions
/// @dev Contains utility functions for calling gas efficiently dataFeedStore functions and decoding return data
library Blocksense {
  function _getLatestSingleFeedData(
    address dataFeedStore,
    uint256 id
  ) internal view returns (bytes32) {
    return
      _callSingleDataFeed(dataFeedStore, (uint256(0x82) << 248) | (id << 120));
  }

  function _getLatestFeedData(
    address dataFeedStore,
    uint256 stride,
    uint256 id
  ) internal view returns (bytes32[] memory) {
    return
      _callDataFeed(
        dataFeedStore,
        (uint256(0x84) << 248) | (stride << 240) | (id << 120),
        17,
        1 << stride
      );
  }

  function _getLatestSlicedFeedData(
    address dataFeedStore,
    uint256 stride,
    uint256 id,
    uint256 startSlot,
    uint256 slotsCount // if this value is 0, then it will return all the slots for the feed starting at startSlot
  ) internal view returns (bytes32[] memory) {
    return
      _callDataFeed(
        dataFeedStore,
        (uint256(0x84) << 248) |
          (stride << 240) |
          (id << 120) |
          (startSlot << 88) |
          (slotsCount << 56),
        slotsCount == 0 ? 21 : 25,
        slotsCount > 0 ? slotsCount : (1 << (stride - startSlot))
      );
  }

  /// @notice Gets the single feed data at a given round ID from the dataFeedStore
  /// @param id The key ID for the feed
  /// @param _round The round ID to retrieve data for
  /// @param dataFeedStore The address of the dataFeedStore contract
  /// @return data The value stored for the feed at the given round ID
  function _getSingleFeedDataAtRound(
    address dataFeedStore,
    uint256 id,
    uint256 _round
  ) internal view returns (bytes32) {
    return
      _callSingleDataFeed(
        dataFeedStore,
        // 1st 2 bytes are function selector and stride (which is always 0 for CL adapters)
        // after that are 15 bytes of the feed id
        // after the feed id are 2 bytes of the round id
        (uint256(0x86) << 248) | (id << 120) | (_round << 104)
      );
  }

  function _getFeedDataAtRound(
    address dataFeedStore,
    uint256 stride,
    uint256 id,
    uint256 _round
  ) internal view returns (bytes32[] memory) {
    return
      _callDataFeed(
        dataFeedStore,
        (uint256(0x86) << 248) |
          (stride << 240) |
          (id << 120) |
          (_round << 104),
        19,
        1 << stride
      );
  }

  function _getSlicedFeedDataAtRound(
    address dataFeedStore,
    uint256 stride,
    uint256 id,
    uint256 _round,
    uint256 startSlot,
    uint256 slotsCount // if this value is 0, then it will return all the slots for the feed starting at startSlot
  ) internal view returns (bytes32[] memory) {
    return
      _callDataFeed(
        dataFeedStore,
        (uint256(0x86) << 248) |
          (stride << 240) |
          (id << 120) |
          (_round << 104) |
          (startSlot << 72) |
          (slotsCount << 40),
        slotsCount == 0 ? 23 : 27,
        slotsCount > 0 ? slotsCount : (1 << (stride - startSlot))
      );
  }

  /// @notice Gets the latest round ID for a given feed from the dataFeedStore
  /// @dev Using assembly achieves lower gas costs
  /// @param id The key ID for the feed
  /// @param dataFeedStore The address of the dataFeedStore contract
  /// @return round The latest round ID
  function _getLatestRound(
    address dataFeedStore,
    uint256 stride,
    uint256 id
  ) internal view returns (uint256) {
    return
      uint256(
        // 1st 2 bytes are function selector and stride (which is always 0 for CL adapters)
        // after that are 15 bytes of the feed id
        _callSingleDataFeed(
          dataFeedStore,
          (uint256(0x81) << 248) | (stride << 240) | (id << 120)
        )
      );
  }

  /// @notice Gets the latest single feed data for a given feed and its latest round ID from the dataFeedStore
  /// @dev Using assembly achieves lower gas costs
  /// @param id The key ID for the feed
  /// @param dataFeedStore The address of the dataFeedStore contract
  /// @return data The latest stored value
  /// @return round The latest round ID
  function _getLatestSingleFeedDataAndRound(
    address dataFeedStore,
    uint256 id
  ) internal view returns (bytes32 data, uint256 round) {
    // using assembly staticcall costs less gas than using a view function
    assembly {
      // get free memory pointer
      let ptr := mload(0x40)

      // store selector in memory at location 0
      // 1st 2 bytes are function selector and stride (which is always 0 for CL adapters)
      // after that are 15 bytes of the feed id
      mstore(
        0x00,
        or(
          0x8300000000000000000000000000000000000000000000000000000000000000,
          shl(120, id)
        )
      )

      // call dataFeedStore with selector and store return value (64 bytes) at memory location ptr
      let success := staticcall(gas(), dataFeedStore, 0x00, 17, ptr, 64)

      // revert if call failed
      if iszero(success) {
        revert(0, 0)
      }

      // load return value from memory at location ptr
      // round is stored in the first 32 bytes of the returned 64 bytes
      round := mload(ptr)

      // value is stored in the second 32 bytes of the returned 64 bytes
      data := mload(add(ptr, 32))
    }
  }

  function _getLatestFeedDataAndRound(
    address dataFeedStore,
    uint256 stride,
    uint256 id
  ) internal view returns (bytes32[] memory data, uint256 round) {
    return
      _callDataFeedAndRound(
        dataFeedStore,
        (uint256(0x85) << 248) | (stride << 240) | (id << 120),
        17,
        1 << stride
      );
  }

  function _getLatestSlicedFeedDataAndRound(
    address dataFeedStore,
    uint256 stride,
    uint256 id,
    uint256 startSlot,
    uint256 slotsCount // if this value is 0, then it will return all the slots for the feed starting at startSlot
  ) internal view returns (bytes32[] memory data, uint256 round) {
    return
      _callDataFeedAndRound(
        dataFeedStore,
        (uint256(0x85) << 248) |
          (stride << 240) |
          (id << 120) |
          (startSlot << 88) |
          (slotsCount << 56),
        slotsCount == 0 ? 23 : 27,
        slotsCount > 0 ? slotsCount : (1 << (stride - startSlot))
      );
  }

  /// @notice Calls the dataFeedStore with the given data
  /// @dev Using assembly achieves lower gas costs
  /// Used as a call() function to dataFeedStore
  /// @param dataFeedStore The address of the dataFeedStore contract
  /// @param selector The data to call the dataFeedStore with
  /// @return returnData The return value from the dataFeedStore
  function _callSingleDataFeed(
    address dataFeedStore,
    uint256 selector
  ) internal view returns (bytes32 returnData) {
    // using assembly staticcall costs less gas than using a view function
    assembly {
      // get free memory pointer
      let ptr := mload(0x40)

      // store selector in memory at location 0
      mstore(0x00, selector)

      // call dataFeedStore with data and store return value (32 bytes) at memory location ptr
      let success := staticcall(
        gas(), // gas remaining
        dataFeedStore, // address to call
        0x00, // location of data to call
        19, // size of data to call - usually it is 17b but for _getRoundData it is 19b because of the 2 bytes of the round
        ptr, // where to store the return data
        32 // how much data to store
      )

      // revert if call failed
      if iszero(success) {
        revert(0, 0)
      }

      // assign loaded return value at memory location ptr to returnData
      returnData := mload(ptr)
    }
  }

  function _callDataFeed(
    address dataFeedStore,
    uint256 selector,
    uint256 selectorLength,
    uint256 length
  ) internal view returns (bytes32[] memory returnData) {
    returnData = new bytes32[](length);
    // using assembly staticcall costs less gas than using a view function
    assembly {
      // get free memory pointer
      let ptr := mload(0x40)
      length := shl(5, length)

      // store selector in memory at location 0
      mstore(0x00, selector)

      // call dataFeedStore with data and store return value (32 bytes) at memory location ptr
      let success := staticcall(
        gas(), // gas remaining
        dataFeedStore, // address to call
        0x00, // location of data to call
        selectorLength, // size of data to call - usually it is 19b but for _getRoundData it is 21b because of the 2 bytes of the round
        ptr, // where to store the return data
        length // how much data to store
      )

      // revert if call failed
      if iszero(success) {
        revert(0, 0)
      }

      let array := add(returnData, 32)

      // assign loaded return value at memory location ptr to returnData
      for {
        let i := 0
      } lt(i, length) {
        i := add(i, 32)
      } {
        mstore(add(array, i), mload(add(ptr, i)))
      }
    }
  }

  function _callDataFeedAndRound(
    address dataFeedStore,
    uint256 selector,
    uint256 selectorLength,
    uint256 length
  ) internal view returns (bytes32[] memory returnData, uint256 round) {
    returnData = new bytes32[](length);
    // using assembly staticcall costs less gas than using a view function
    assembly {
      // get free memory pointer
      let ptr := mload(0x40)
      length := add(shl(5, length), 32)

      // store selector in memory at location 0
      mstore(0x00, selector)

      // call dataFeedStore with data and store return value (32 bytes) at memory location ptr
      let success := staticcall(
        gas(), // gas remaining
        dataFeedStore, // address to call
        0x00, // location of data to call
        selectorLength, // size of data to call - usually it is 19b but for _getRoundData it is 21b because of the 2 bytes of the round
        ptr, // where to store the return data
        length // how much data to store
      )

      // revert if call failed
      if iszero(success) {
        revert(0, 0)
      }

      // load return value from memory at location ptr
      // round is stored in the first 32 bytes of the returned 64 bytes
      round := mload(ptr)

      // assign loaded return value at memory location ptr to returnData
      for {
        let i := 32
      } gt(length, i) {
        i := add(i, 32)
      } {
        mstore(add(returnData, i), mload(add(ptr, i)))
      }
    }
  }

  /// @notice Decodes the return data from the dataFeedStore
  /// This function can be used to separate the value and timestamp from the return data
  /// This is useful for feeds that return prices
  /// @param data The data to decode
  /// @return answer The value stored for the feed at the given round ID
  /// @return timestamp The timestamp when the value was stored
  function _decodeData(bytes32 data) internal pure returns (uint256, uint256) {
    return (uint256(uint192(bytes24(data))), uint64(uint256(data)));
  }
}
