// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

// array[key] -> array[counter] -> Transmission {bytes24 value, uint64 timestamp}

/// @title Array of arrays Historic Data Feed Storage
contract HistoricDataFeedStoreV1 {
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GETTER =
    0x00000000000000000000000000000000000000000000000000000000E0000000;
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_LATEST =
    0x0000000000000000000000000000000000000000000000000000000080000000;
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_LATEST_COUNTER =
    0x0000000000000000000000000000000000000000000000000000000040000000;
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_AT_COUNTER =
    0x0000000000000000000000000000000000000000000000000000000020000000;
  uint16 internal constant MAX_COUNTER = 0xFFFF;

  /// @notice Owner of the contract
  /// @dev The owner is the address that deployed the contract
  address internal immutable owner;

  constructor() {
    owner = msg.sender;
  }

  /// @notice Fallback function
  /// @dev The fallback function is used to set or get data feeds according to the provided selector.
  fallback() external {
    // getters
    assembly {
      // store selector in memory at location 28
      calldatacopy(0x1C, 0x00, 0x04)
      let selector := mload(0x00)

      let key := and(selector, not(CONTRACT_MANAGEMENT_SELECTOR_GETTER))

      // getFeedById(uint32 key) returns (bytes32)
      if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_LATEST) {
        let position := mul(key, MAX_COUNTER)
        mstore(0x00, sload(add(position, sload(position))))

        // return value stored at memory location 0
        return(0x00, 0x20)
      }
      // getLatestCounter(uint32 key) returns (uint16)
      if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_LATEST_COUNTER) {
        // load value at array[key] and store it at memory location 0
        mstore(0x00, sload(mul(key, MAX_COUNTER)))

        // return value stored at memory location 0
        return(0x00, 0x20)
      }
      // getFeedAtCounter(uint32 key, uint16 counter) returns (bytes32)
      if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_AT_COUNTER) {
        let position := mul(key, MAX_COUNTER)
        // load value at array[key] and store it at memory location 0
        mstore(0x00, sload(add(position, calldataload(0x04))))

        // return value stored at memory location 0
        return(0x00, 0x20)
      }
    }

    address _owner = owner;

    // setters
    assembly {
      // check if sender is owner
      if iszero(eq(_owner, caller())) {
        revert(0, 0)
      }

      let selector := mload(0x00)

      // setFeeds(bytes)
      if eq(selector, 0x1a2d80ac) {
        // bytes should be in the format of:
        // <key1><value1>...<keyN><valueN>
        // where key is uint32 and value is bytes32
        let time := timestamp()
        let len := calldatasize()
        for {
          // start at location 0x04 where first key is stored after the selector
          let i := 4
        } lt(i, len) {
          // increment by 36 bytes (4 bytes for key and 32 bytes for value)
          i := add(i, 0x24)
        } {
          // store key in memory at the last 4 bytes of the second slot 60-64
          calldatacopy(0x3C, i, 0x04)
          let key := mload(0x20)
          let counterPos := mul(key, MAX_COUNTER)
          let counter := sload(counterPos)
          let currentCounter := addmod(counter, 1, MAX_COUNTER)
          if iszero(currentCounter) {
            currentCounter := 1
          }

          sstore(counterPos, currentCounter)

          // store value in storage at slot key (index)
          sstore(
            add(counterPos, currentCounter),
            or(calldataload(add(i, 0x04)), time)
          )
        }
        return(0, 0)
      }
      revert(0, 0)
    }
  }
}
