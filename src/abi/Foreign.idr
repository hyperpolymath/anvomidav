-- SPDX-License-Identifier: PMPL-1.0-or-later
||| ANVOMIDAV — FFI Bridge Declarations
|||
||| This module defines the formal bridge to the native C/Zig implementation.
||| It wraps raw FFI calls in safe, total Idris functions.

module ANVOMIDAV.ABI.Foreign

import ANVOMIDAV.ABI.Types
import ANVOMIDAV.ABI.Layout

%default total

--------------------------------------------------------------------------------
-- Lifecycle
--------------------------------------------------------------------------------

||| Initializes the Anvomidav kernel.
export
%foreign "C:anvomidav_init, libanvomidav"
prim__init : PrimIO Bits64

||| Safe initialization wrapper. Returns a managed Handle.
export
init : IO (Maybe Handle)
init = do
  ptr <- primIO prim__init
  pure (createHandle ptr)

||| Releases Anvomidav kernel resources.
export
%foreign "C:anvomidav_free, libanvomidav"
prim__free : Bits64 -> PrimIO ()

||| Safe cleanup wrapper.
export
free : Handle -> IO ()
free h = primIO (prim__free (handlePtr h))
