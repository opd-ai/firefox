/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// CONTRACT: blake2b_kat API Surface
//
// Header-only file defining BLAKE2b Known Answer Test (KAT) vectors.
// Source: https://github.com/BLAKE2/BLAKE2/blob/master/testvectors/blake2b-kat.txt
//
// Global constants:
//   const std::vector<uint8_t> kat_key
//     - 64-byte key (values 0x00–0x3F) used for keyed-mode KAT vectors
//
//   const std::vector<uint8_t> kat_data
//     - 256-byte message corpus (values 0x00–0xFF); test[i] hashes kat_data[0..i]
//
//   std::vector<std::pair<int, std::vector<uint8_t>>> TestcasesUnkeyed
//     - 256 entries; pair<message_length, expected_blake2b512_hash(64 bytes)>
//     - Computed without a key; message_length == entry index
//
//   std::vector<std::pair<int, std::vector<uint8_t>>> TestcasesKeyed
//     - 256 entries; pair<message_length, expected_blake2b512_hash(64 bytes)>
//     - Computed with kat_key; message_length == entry index
//
// Invariants:
//   kat_key.size()          == 64
//   kat_data.size()         == 256
//   TestcasesUnkeyed.size() == 256   (one entry per message length 0..255)
//   TestcasesKeyed.size()   == 256   (one entry per message length 0..255)
//   All hash vectors        have size 64  (BLAKE2b-512 = 64 bytes)
//   TestcasesUnkeyed[i].first == i
//   TestcasesKeyed[i].first   == i

#include "blake2b_kat.h"

#include <cassert>
#include <cstdio>
#include <cstdlib>

// Known first unkeyed hash (empty message):
// BLAKE2b-512("") = 786a02f7...
static const uint8_t kFirstUnkeyed[64] = {
    0x78, 0x6a, 0x02, 0xf7, 0x42, 0x01, 0x59, 0x03, 0xc6, 0xc6, 0xfd,
    0x85, 0x25, 0x52, 0xd2, 0x72, 0x91, 0x2f, 0x47, 0x40, 0xe1, 0x58,
    0x47, 0x61, 0x8a, 0x86, 0xe2, 0x17, 0xf7, 0x1f, 0x54, 0x19, 0xd2,
    0x5e, 0x10, 0x31, 0xaf, 0xee, 0x58, 0x53, 0x13, 0x89, 0x64, 0x44,
    0x93, 0x4e, 0xb0, 0x4b, 0x90, 0x3a, 0x68, 0x5b, 0x14, 0x48, 0xb7,
    0x55, 0xd5, 0x6f, 0x70, 0x1a, 0xfe, 0x9b, 0xe2, 0xce};

// Known first keyed hash (empty message + kat_key):
// BLAKE2b-512("", key=kat_key) = 10ebb677...
static const uint8_t kFirstKeyed[64] = {
    0x10, 0xeb, 0xb6, 0x77, 0x00, 0xb1, 0x86, 0x8e, 0xfb, 0x44, 0x17,
    0x98, 0x7a, 0xcf, 0x46, 0x90, 0xae, 0x9d, 0x97, 0x2f, 0xb7, 0xa5,
    0x90, 0xc2, 0xf0, 0x28, 0x71, 0x79, 0x9a, 0xaa, 0x47, 0x86, 0xb5,
    0xe9, 0x96, 0xe8, 0xf0, 0xf4, 0xeb, 0x98, 0x1f, 0xc2, 0x14, 0xb0,
    0x05, 0xf4, 0x2d, 0x2f, 0xf4, 0x23, 0x34, 0x99, 0x39, 0x16, 0x53,
    0xdf, 0x7a, 0xef, 0xcb, 0xc1, 0x3f, 0xc5, 0x15, 0x68};

static void check(bool condition, const char* msg) {
  if (!condition) {
    fprintf(stderr, "FAIL: %s\n", msg);
    exit(1);
  }
}

int main(void) {
  // kat_key: 64 bytes, sequential values 0x00..0x3F
  check(kat_key.size() == 64, "kat_key.size() == 64");
  for (size_t i = 0; i < 64; ++i) {
    check(kat_key[i] == static_cast<uint8_t>(i), "kat_key[i] == i");
  }

  // kat_data: 256 bytes, sequential values 0x00..0xFF
  check(kat_data.size() == 256, "kat_data.size() == 256");
  for (size_t i = 0; i < 256; ++i) {
    check(kat_data[i] == static_cast<uint8_t>(i), "kat_data[i] == i");
  }

  // TestcasesUnkeyed: 256 entries, each with a 64-byte hash
  check(TestcasesUnkeyed.size() == 256, "TestcasesUnkeyed.size() == 256");
  for (size_t i = 0; i < TestcasesUnkeyed.size(); ++i) {
    check(TestcasesUnkeyed[i].first == static_cast<int>(i),
          "TestcasesUnkeyed[i].first == i");
    check(TestcasesUnkeyed[i].second.size() == 64,
          "TestcasesUnkeyed[i].second.size() == 64");
  }

  // Spot-check first unkeyed vector (empty message)
  const auto& uv0 = TestcasesUnkeyed[0].second;
  for (size_t i = 0; i < 64; ++i) {
    check(uv0[i] == kFirstUnkeyed[i],
          "TestcasesUnkeyed[0] matches known empty-message hash");
  }

  // TestcasesKeyed: 256 entries, each with a 64-byte hash
  check(TestcasesKeyed.size() == 256, "TestcasesKeyed.size() == 256");
  for (size_t i = 0; i < TestcasesKeyed.size(); ++i) {
    check(TestcasesKeyed[i].first == static_cast<int>(i),
          "TestcasesKeyed[i].first == i");
    check(TestcasesKeyed[i].second.size() == 64,
          "TestcasesKeyed[i].second.size() == 64");
  }

  // Spot-check first keyed vector (empty message + kat_key)
  const auto& kv0 = TestcasesKeyed[0].second;
  for (size_t i = 0; i < 64; ++i) {
    check(kv0[i] == kFirstKeyed[i],
          "TestcasesKeyed[0] matches known keyed empty-message hash");
  }

  // Keyed and unkeyed hashes must differ for the same input
  check(uv0 != kv0, "Unkeyed and keyed hashes for empty message differ");

  printf("PASS: all blake2b_kat contract tests passed\n");
  return 0;
}
