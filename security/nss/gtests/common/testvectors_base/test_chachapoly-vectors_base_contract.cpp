// CONTRACT: chachapoly-vectors_base API Surface
//
// File:    security/nss/gtests/common/testvectors_base/chachapoly-vectors_base.h
// Type:    Header-only — defines a global const array of ChaCha20/Poly1305 test vectors
// Source:  RFC 7539 (ChaCha20/Poly1305 AEAD)
//
// Public API:
//   const ChaChaTestVector kChaCha20Vectors[]
//     — Array of 2 ChaCha20/Poly1305 AEAD test vectors from RFC 7539.
//     — Each element contains plaintext, aad, key, iv, ciphertext byte arrays,
//       plus invalid_tag and invalid_iv flags.
//
// ChaChaTestVector fields (from test-structs.h):
//   uint32_t id                    — sequential test identifier
//   std::vector<uint8_t> plaintext — plaintext bytes
//   std::vector<uint8_t> aad       — additional authenticated data
//   std::vector<uint8_t> key       — 32-byte ChaCha20 key
//   std::vector<uint8_t> iv        — 12-byte nonce / IV
//   std::vector<uint8_t> ciphertext— authenticated ciphertext
//   bool invalid_tag               — true if tag should fail verification
//   bool invalid_iv                — true if IV is malformed/invalid
//
// Vector 0 — RFC 7539 §2.8.2: normal encrypt/decrypt
// Vector 1 — RFC 7539 Appendix A.5: normal encrypt/decrypt

// Self-contained: define the minimal ChaChaTestVector struct without pulling
// in the full NSS/NSPR dependency chain (which requires a mach build env).
// The struct layout is identical to ChaChaTestVectorStr in test-structs.h.

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <vector>

// Minimal reproduction of ChaChaTestVectorStr from test-structs.h.
// Field names and types are preserved exactly so that
// chachapoly-vectors_base.h can be included unchanged.
typedef struct ChaChaTestVectorStr {
  uint32_t id;
  std::vector<uint8_t> plaintext;
  std::vector<uint8_t> aad;
  std::vector<uint8_t> key;
  std::vector<uint8_t> iv;
  std::vector<uint8_t> ciphertext;
  bool invalid_tag;
  bool invalid_iv;
} ChaChaTestVector;

// Include the header under test — it defines kChaCha20Vectors[].
#include "chachapoly-vectors_base.h"

// ---------------------------------------------------------------------------
// Helper: count elements of kChaCha20Vectors (file-scope array)
// ---------------------------------------------------------------------------
static constexpr std::size_t kVectorCount =
    sizeof(kChaCha20Vectors) / sizeof(kChaCha20Vectors[0]);

// ---------------------------------------------------------------------------
// Contract tests
// ---------------------------------------------------------------------------

static void test_vector_count() {
  // RFC 7539 provides exactly 2 ChaCha20/Poly1305 test vectors.
  assert(kVectorCount == 2 && "kChaCha20Vectors must have exactly 2 entries");
}

static void test_vector_ids_are_sequential() {
  assert(kChaCha20Vectors[0].id == 0 && "first vector id must be 0");
  assert(kChaCha20Vectors[1].id == 1 && "second vector id must be 1");
}

static void test_validity_flags_are_false() {
  // Both RFC 7539 vectors are valid — no tag or IV errors expected.
  for (std::size_t i = 0; i < kVectorCount; ++i) {
    assert(!kChaCha20Vectors[i].invalid_tag &&
           "RFC 7539 test vectors must have invalid_tag == false");
    assert(!kChaCha20Vectors[i].invalid_iv &&
           "RFC 7539 test vectors must have invalid_iv == false");
  }
}

static void test_key_length() {
  // ChaCha20 uses a 256-bit (32-byte) key.
  for (std::size_t i = 0; i < kVectorCount; ++i) {
    assert(kChaCha20Vectors[i].key.size() == 32 &&
           "ChaCha20 key must be 32 bytes");
  }
}

static void test_iv_length() {
  // ChaCha20/Poly1305 uses a 96-bit (12-byte) nonce.
  for (std::size_t i = 0; i < kVectorCount; ++i) {
    assert(kChaCha20Vectors[i].iv.size() == 12 &&
           "ChaCha20/Poly1305 IV must be 12 bytes");
  }
}

static void test_ciphertext_at_least_as_long_as_plaintext() {
  // Authenticated encryption produces ciphertext >= plaintext length
  // (the Poly1305 tag is 16 bytes appended separately in NSS).
  for (std::size_t i = 0; i < kVectorCount; ++i) {
    assert(kChaCha20Vectors[i].ciphertext.size() >=
               kChaCha20Vectors[i].plaintext.size() &&
           "ciphertext must be at least as long as plaintext");
  }
}

// Vector 0 — RFC 7539 §2.8.2 known-good values
static void test_vector0_known_values() {
  const ChaChaTestVector& v = kChaCha20Vectors[0];

  // Plaintext: "Ladies and Gentlemen of the class of '99: ..."
  assert(v.plaintext.size() == 114 && "vector 0 plaintext must be 114 bytes");
  assert(v.plaintext[0] == 0x4c && "vector 0 plaintext[0] must be 0x4c ('L')");

  // AAD: 0x50 0x51 0x52 0x53 0xc0 0xc1 0xc2 0xc3 0xc4 0xc5 0xc6 0xc7
  assert(v.aad.size() == 12 && "vector 0 AAD must be 12 bytes");
  assert(v.aad[0] == 0x50 && "vector 0 aad[0] must be 0x50");
  assert(v.aad[3] == 0x53 && "vector 0 aad[3] must be 0x53");

  // Key: 0x80..0x9f (32 bytes)
  assert(v.key[0] == 0x80 && "vector 0 key[0] must be 0x80");
  assert(v.key[31] == 0x9f && "vector 0 key[31] must be 0x9f");

  // IV: 0x07 0x00 0x00 0x00 0x40 0x41 0x42 0x43 0x44 0x45 0x46 0x47
  assert(v.iv[0] == 0x07 && "vector 0 iv[0] must be 0x07");
  assert(v.iv[4] == 0x40 && "vector 0 iv[4] must be 0x40");

  // Ciphertext = plaintext (114 bytes) + Poly1305 tag (16 bytes) = 130 bytes
  assert(v.ciphertext[0] == 0xd3 && "vector 0 ciphertext[0] must be 0xd3");
  assert(v.ciphertext.size() == 130 &&
         "vector 0 ciphertext must be 130 bytes (114 plaintext + 16 tag)");
}

// Vector 1 — RFC 7539 Appendix A.5 known-good values
static void test_vector1_known_values() {
  const ChaChaTestVector& v = kChaCha20Vectors[1];

  // Plaintext: "Internet-Drafts are draft documents ..."
  assert(v.plaintext.size() == 265 && "vector 1 plaintext must be 265 bytes");
  assert(v.plaintext[0] == 0x49 &&
         "vector 1 plaintext[0] must be 0x49 ('I')");

  // AAD: 0xf3 0x33 ...
  assert(v.aad.size() == 12 && "vector 1 AAD must be 12 bytes");
  assert(v.aad[0] == 0xf3 && "vector 1 aad[0] must be 0xf3");

  // Key: 0x1c 0x92 ...
  assert(v.key[0] == 0x1c && "vector 1 key[0] must be 0x1c");

  // IV: 0x00 0x00 0x00 0x00 0x01 0x02 ...
  assert(v.iv[0] == 0x00 && "vector 1 iv[0] must be 0x00");
  assert(v.iv[4] == 0x01 && "vector 1 iv[4] must be 0x01");

  // Ciphertext
  assert(v.ciphertext[0] == 0x64 && "vector 1 ciphertext[0] must be 0x64");
  assert(v.ciphertext.size() == 281 &&
         "vector 1 ciphertext must be 281 bytes");
}

int main() {
  test_vector_count();
  test_vector_ids_are_sequential();
  test_validity_flags_are_false();
  test_key_length();
  test_iv_length();
  test_ciphertext_at_least_as_long_as_plaintext();
  test_vector0_known_values();
  test_vector1_known_values();

  printf("All contract tests passed for chachapoly-vectors_base.\n");
  return 0;
}
