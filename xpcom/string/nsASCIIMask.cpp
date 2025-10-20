/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=80: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "nsASCIIMask.h"

namespace mozilla {

#ifdef MOZ_RUST_ASCIIMASK
// Use Rust implementation

extern "C" {
  const ASCIIMaskArray* ASCIIMask_MaskWhitespace();
  const ASCIIMaskArray* ASCIIMask_MaskCRLF();
  const ASCIIMaskArray* ASCIIMask_MaskCRLFTab();
  const ASCIIMaskArray* ASCIIMask_Mask0to9();
}

const ASCIIMaskArray& ASCIIMask::MaskWhitespace() {
  return *ASCIIMask_MaskWhitespace();
}

const ASCIIMaskArray& ASCIIMask::MaskCRLF() {
  return *ASCIIMask_MaskCRLF();
}

const ASCIIMaskArray& ASCIIMask::MaskCRLFTab() {
  return *ASCIIMask_MaskCRLFTab();
}

const ASCIIMaskArray& ASCIIMask::Mask0to9() {
  return *ASCIIMask_Mask0to9();
}

#else
// Use C++ implementation

constexpr bool TestWhitespace(char c) {
  return c == '\f' || c == '\t' || c == '\r' || c == '\n' || c == ' ';
}
constexpr ASCIIMaskArray sWhitespaceMask = CreateASCIIMask(TestWhitespace);

constexpr bool TestCRLF(char c) { return c == '\r' || c == '\n'; }
constexpr ASCIIMaskArray sCRLFMask = CreateASCIIMask(TestCRLF);

constexpr bool TestCRLFTab(char c) {
  return c == '\r' || c == '\n' || c == '\t';
}
constexpr ASCIIMaskArray sCRLFTabMask = CreateASCIIMask(TestCRLFTab);

constexpr bool TestZeroToNine(char c) {
  return c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' ||
         c == '6' || c == '7' || c == '8' || c == '9';
}
constexpr ASCIIMaskArray sZeroToNineMask = CreateASCIIMask(TestZeroToNine);

const ASCIIMaskArray& ASCIIMask::MaskWhitespace() { return sWhitespaceMask; }

const ASCIIMaskArray& ASCIIMask::MaskCRLF() { return sCRLFMask; }

const ASCIIMaskArray& ASCIIMask::MaskCRLFTab() { return sCRLFTabMask; }

const ASCIIMaskArray& ASCIIMask::Mask0to9() { return sZeroToNineMask; }

#endif  // MOZ_RUST_ASCIIMASK

}  // namespace mozilla
