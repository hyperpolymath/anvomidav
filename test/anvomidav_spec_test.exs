# SPDX-License-Identifier: MIT OR PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath)

# Anvomidav — Figure Skating Choreography DSL
#
# Placeholder test module for the Anvomidav language.
# Anvomidav is currently in the concept phase (no implementation yet).
# These tests validate the planned domain model and will grow as the
# compiler/interpreter is built out.
#
# The test file uses ExUnit (Elixir) as a lightweight harness that
# does not require a full build system during the concept phase.

ExUnit.start()

defmodule Anvomidav.SpecTest do
  @moduledoc """
  Specification tests for the Anvomidav figure skating DSL.

  These tests encode domain invariants from the ISU (International Skating
  Union) technical rules, ensuring the language design respects real-world
  constraints from day one.
  """
  use ExUnit.Case, async: true

  # ------------------------------------------------------------------
  # ISU Element Categories
  # ------------------------------------------------------------------

  @isu_jump_types ~w(toe_loop salchow loop flip lutz axel)
  @isu_spin_types ~w(upright sit camel combination)
  @isu_max_jumping_passes_short 3
  @isu_max_jumping_passes_free 7

  test "ISU jump catalogue contains the six recognised jumps" do
    assert length(@isu_jump_types) == 6
    assert "axel" in @isu_jump_types
    assert "lutz" in @isu_jump_types
  end

  test "ISU spin catalogue contains the four recognised categories" do
    assert length(@isu_spin_types) == 4
    assert "combination" in @isu_spin_types
  end

  test "short programme allows at most 3 jumping passes" do
    programme_jumps = [:toe_loop, :salchow, :axel]
    assert length(programme_jumps) <= @isu_max_jumping_passes_short
  end

  test "free programme allows at most 7 jumping passes" do
    programme_jumps = [:lutz, :flip, :loop, :salchow, :toe_loop, :axel, :lutz]
    assert length(programme_jumps) <= @isu_max_jumping_passes_free
  end

  test "exceeding maximum jumping passes is detected" do
    programme_jumps = List.duplicate(:axel, 8)
    assert length(programme_jumps) > @isu_max_jumping_passes_free
  end

  # ------------------------------------------------------------------
  # Planned Notation Primitives
  # ------------------------------------------------------------------

  test "element notation is a {type, name, level} triple" do
    element = %{type: :jump, name: :axel, level: 2}
    assert element.type == :jump
    assert element.name == :axel
    assert element.level == 2
  end

  test "sequence is an ordered list of elements" do
    sequence = [
      %{type: :jump, name: :lutz, level: 3},
      %{type: :spin, name: :camel, level: 4},
      %{type: :step, name: :step_sequence, level: 3}
    ]
    assert length(sequence) == 3
    assert hd(sequence).type == :jump
  end
end
