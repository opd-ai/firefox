/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=80: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "mozilla/Unused.h"

#ifdef MOZ_RUST_UNUSED

// Rust implementation
// The Rust port exports mozilla_Unused as extern "C"
// We create a reference to it in the mozilla namespace
extern "C" {
  extern const mozilla::unused_t mozilla_Unused;
}

namespace mozilla {
  // Make Rust-exported symbol available as mozilla::Unused
  const unused_t& Unused = mozilla_Unused;
}  // namespace mozilla

#else

// C++ implementation
namespace mozilla {

const unused_t Unused = unused_t();

}  // namespace mozilla

#endif  // MOZ_RUST_UNUSED
