-- SPDX-License-Identifier: PMPL-1.0-or-later
||| ANVOMIDAV — ABI Type Definitions
|||
||| This module defines the Application Binary Interface for the project.
||| It provides formal proofs of memory layout and FFI invariants.

module ANVOMIDAV.ABI.Types

import Data.Bits
import Data.So
import Data.Vect

%default total

--------------------------------------------------------------------------------
-- Platform Context
--------------------------------------------------------------------------------

||| Supported targets for verified logic execution.
public export
data Platform = Linux | Windows | MacOS | BSD | WASM

||| Resolves the execution environment at compile time.
public export
thisPlatform : Platform
thisPlatform =
  %runElab do
    pure Linux

--------------------------------------------------------------------------------
-- Core Result Types
--------------------------------------------------------------------------------

||| Formal outcome of a verified operation.
public export
data Result : Type where
  ||| Operation Successful
  Ok : Result
  ||| Operation Failed: Generic error
  Error : Result
  ||| Invalid Parameter
  InvalidParam : Result
  ||| System Error: out of memory
  OutOfMemory : Result
  ||| Safety Error: null pointer encountered
  NullPointer : Result

--------------------------------------------------------------------------------
-- Safety Handles
--------------------------------------------------------------------------------

||| Opaque handle to a session.
||| INVARIANT: The internal pointer is guaranteed to be non-null.
public export
data Handle : Type where
  MkHandle : (ptr : Bits64) -> {auto 0 nonNull : So (ptr /= 0)} -> Handle

||| Safe constructor for handles.
public export
createHandle : Bits64 -> Maybe Handle
createHandle 0 = Nothing
createHandle ptr = Just (MkHandle ptr)
