// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title Mapping of mappings Historic Data Feed Storage
/// @dev Storage layout
///
/// mapping(key => mapping(counter => Transmission {bytes24 value, uint64 timestamp}))
/// array[key] -> latestCounter
/// We have a mapping of mappings where:
/// - key is the feed id and mapping[key] is the mapping of counters;
/// - counter is incremented whenever a new value for a feed is stored
///   and presents a continuous feed history.
/// Latest counter is stored at array[key].
/// Each counter mapping is stored at keccak256(key) address.
/// The value is then stored at keccak256(key) + counter.
contract HistoricDataFeedStoreV2 {
  /// @notice Mask for getters
  /// @dev The maximum length of the key is defined by the mask from the selector.
  /// This evaluates to 29 bits, which is the maximum length of the key.
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GETTER =
    0x00000000000000000000000000000000000000000000000000000000E0000000;

  /// @notice Mask for getFeedById(uint32 key)
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_LATEST =
    0x0000000000000000000000000000000000000000000000000000000080000000;

  /// @notice Mask for getLatestCounter(uint32 key)
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_LATEST_COUNTER =
    0x0000000000000000000000000000000000000000000000000000000040000000;

  /// @notice Mask for getFeedAtCounter(uint32 key, uint256 counter)
  bytes32 internal constant CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_AT_COUNTER =
    0x0000000000000000000000000000000000000000000000000000000020000000;

  /// @notice Maximum value for the counter
  /// @dev When the counter reaches the maximum value, it resets to one.
  /// This is to have less gas cost when the value is incremented again.
  uint16 internal constant MAX_COUNTER = 0xFFFF;

  /// @notice Owner of the contract
  /// @dev The owner is the address that deployed the contract
  address internal immutable owner;

  /// @notice Constructor
  /// @dev Sets the owner of the contract - the address that deployed the contract
  constructor() {
    owner = msg.sender;
  }

  /// @notice Fallback function
  /// @dev The fallback function is used to set or get data feeds according to the provided selector.
  fallback() external {
    bytes32 selector;

    // Getters
    assembly {
      // Get the address of the first free memory slot
      let ptr := mload(0x40)

      // Store selector in memory at location free memory slot + 28 bytes
      calldatacopy(add(ptr, 0x1C), 0x00, 0x04)

      // Load selector from memory
      selector := mload(ptr)

      if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GETTER) {
        // Key is the first 4 bytes of the calldata after the selector
        let key := and(selector, not(CONTRACT_MANAGEMENT_SELECTOR_GETTER))

        // getFeedAtCounter(uint32 key, uint256 counter) returns (bytes32 value)
        if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_AT_COUNTER) {
          // Store key in memory at the last 4 bytes of the second slot 60-64
          mstore(0x20, key)

          // keccak256(key) gives the address of the counters mapping
          // providedCounter is the bytes of the calldata after the selector (first 4 bytes)
          // Load value at counters[counter + providedCounter] and store it at memory location 0
          mstore(0x00, sload(add(keccak256(0x3C, 0x04), calldataload(0x04))))

          // Return value stored at memory location 0
          return(0x00, 0x20)
        }

        // Load the latest counter at array[key]
        let counter := sload(key)

        // getFeedById(uint32 key) returns (bytes32 value)
        if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_FEED_LATEST) {
          // Store key in memory at the last 4 bytes of the second slot 60-64
          mstore(0x20, key)

          // keccak256(key) gives the address of the counters mapping
          // then load the value at counters[latest counter] and store it at memory location 0
          mstore(0x00, sload(add(keccak256(0x3C, 0x04), counter)))
        }
        // getLatestCounter(uint32 key) returns (uint256 counter)
        if and(selector, CONTRACT_MANAGEMENT_SELECTOR_GET_LATEST_COUNTER) {
          // Store the counter at array[key] in memory at the last 4 bytes of the first slot 28-32
          mstore(0x20, counter)
        }
        // Return value stored at memory location 0
        return(0x00, 0x40)
      }
    }

    address _owner = owner;

    // Setters
    assembly {
      // Check if sender is owner
      if iszero(eq(_owner, caller())) {
        revert(0x00, 0x00)
      }

      // setFeeds(bytes)
      if eq(selector, 0x1a2d80ac) {
        // Bytes should be in the format of:
        // <key1><value1>...<keyN><valueN>
        // where key is uint32 and value is bytes32

        // Get the current timestamp
        let time := timestamp()

        // Get the length of the calldata
        let len := calldatasize()

        for {
          // Start at location 0x04 where first key is stored after the selector
          let i := 0x04
        } lt(i, len) {
          // Increment by 36 bytes (4 bytes for key and 32 bytes for value)
          i := add(i, 0x24)
        } {
          // Store key in memory at the last 4 bytes of the second slot 60-64
          calldatacopy(0x1C, i, 0x04)

          // Load key from memory
          let key := mload(0x00)

          // Calculate the counters mapping address based on keccak256(key)
          let countersAddress := keccak256(0x1C, 0x04)

          // Load the latest counter and increment it by 1
          let currentCounter := addmod(sload(key), 1, MAX_COUNTER)

          // To avoid getting a zero value in storage when reset to zero,
          // the counter is always one greater than the actual value.
          // This prevents a higher gas cost when the value is incremented again.
          if iszero(currentCounter) {
            currentCounter := 1
          }

          // Store new counter value at array[key]
          sstore(key, currentCounter)

          // Store { value, timestamp } at counters[currentCounter]
          sstore(
            add(countersAddress, currentCounter),
            or(calldataload(add(i, 0x04)), time)
          )
        }
        return(0x00, 0x00)
      }
      // Revert if selector is not recognized.
      revert(0x00, 0x00)
    }
  }
}
