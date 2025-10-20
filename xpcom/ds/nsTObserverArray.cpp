/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=80: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "nsTObserverArray.h"

#ifdef MOZ_RUST_OBSERVER_ARRAY
// Use Rust implementation
extern "C" {
void nsTObserverArray_base_AdjustIterators(void* aThis, size_t aModPos,
                                           ptrdiff_t aAdjustment);
void nsTObserverArray_base_ClearIterators(void* aThis);
}

void nsTObserverArray_base::AdjustIterators(index_type aModPos,
                                            diff_type aAdjustment) {
  nsTObserverArray_base_AdjustIterators(this, aModPos, aAdjustment);
}

void nsTObserverArray_base::ClearIterators() {
  nsTObserverArray_base_ClearIterators(this);
}

#else
// Original C++ implementation
void nsTObserverArray_base::AdjustIterators(index_type aModPos,
                                            diff_type aAdjustment) {
  MOZ_ASSERT(aAdjustment == -1 || aAdjustment == 1, "invalid adjustment");
  Iterator_base* iter = mIterators;
  while (iter) {
    if (iter->mPosition > aModPos) {
      iter->mPosition += aAdjustment;
    }
    iter = iter->mNext;
  }
}

void nsTObserverArray_base::ClearIterators() {
  Iterator_base* iter = mIterators;
  while (iter) {
    iter->mPosition = 0;
    iter = iter->mNext;
  }
}
#endif  // MOZ_RUST_OBSERVER_ARRAY
