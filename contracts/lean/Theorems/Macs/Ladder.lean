/-
  Theorems.Macs.Ladder — L5 machine-checked proofs for the pmat verification
  ladder parser.

  This is the *lighthouse* dogfooding proof for
  `docs/specifications/audit-pmat-support-l1-l5-aprender-provable-contracts.md`:
  the verification ladder proving itself. `VLevel` mirrors
  `VerificationLevel` in `src/cli/handlers/work_verification_level.rs`; the
  theorems below discharge the obligations of the kernel contract
  `contracts/macs-ladder-kernel-v1.yaml`.

  Pure Lean 4 core — no Mathlib dependency, so `lake build` is hermetic and
  fast. Every proof is `by decide` / `cases … <;> decide` and closes its goal
  completely (no proof holes; `#print axioms` reports no dependencies).
-/

namespace Theorems.Macs

/-- The pmat verification ladder as a finite inductive, mirroring the six
    variants of `VerificationLevel`. -/
inductive VLevel where
  | L0 | L1 | L2 | L3 | L4 | L5
deriving DecidableEq, Repr

namespace VLevel

/-- Canonical wire string (mirrors `VerificationLevel::as_str`). -/
def toStr : VLevel → String
  | L0 => "L0" | L1 => "L1" | L2 => "L2"
  | L3 => "L3" | L4 => "L4" | L5 => "L5"

/-- Numeric rung (mirrors the derived `Ord` field order / `as u8`). -/
def toNat : VLevel → Nat
  | L0 => 0 | L1 => 1 | L2 => 2 | L3 => 3 | L4 => 4 | L5 => 5

/-- Strict parser (mirrors `VerificationLevel::parse_strict`). -/
def parseStrict (s : String) : Option VLevel :=
  if s = "L0" then some L0
  else if s = "L1" then some L1
  else if s = "L2" then some L2
  else if s = "L3" then some L3
  else if s = "L4" then some L4
  else if s = "L5" then some L5
  else none

/-- The complete set of accepted input strings. -/
def validStrings : List String := ["L0", "L1", "L2", "L3", "L4", "L5"]

end VLevel

open VLevel

/-- **Ladder_Parse_Total** (obligation `parse_total_strict`): parsing the
    canonical string of any level round-trips to that level — `parseStrict` is
    a total left inverse of `toStr`. -/
theorem Ladder_Parse_Total : ∀ l : VLevel, parseStrict (toStr l) = some l := by
  intro l; cases l <;> decide

/-- **Ladder_Parse_Sound**: a successful parse recovers exactly its input over
    the accepted alphabet (no aliasing between levels). -/
theorem Ladder_Parse_Sound :
    ∀ s ∈ VLevel.validStrings, ∃ l : VLevel, parseStrict s = some l ∧ toStr l = s := by
  decide

/-- **Ladder_Parse_Complete**: every valid string parses (parser is defined on
    the whole accepted alphabet). -/
theorem Ladder_Parse_Complete :
    VLevel.validStrings.all (fun s => (parseStrict s).isSome) = true := by
  decide

/-- **Ladder_Parse_Strict** (obligation `strict_rejects_corruptions`): case,
    whitespace, and out-of-set corruptions are rejected — mirrors the Rust
    falsification tests `parse_strict_rejects_typos`. -/
theorem Ladder_Parse_Strict :
    parseStrict "l3" = none ∧ parseStrict "L3 " = none ∧
    parseStrict " L3" = none ∧ parseStrict "strong" = none ∧
    parseStrict "L6" = none ∧ parseStrict "" = none := by
  decide

/-- **Ladder_Ord_Injective** (obligation `gate_monotone` support): the numeric
    rung is injective, so `Ord`/rung comparisons never conflate two levels. -/
theorem Ladder_Ord_Injective :
    ∀ a b : VLevel, toNat a = toNat b → a = b := by
  intro a b; cases a <;> cases b <;> decide

/-- **Ladder_toStr_Injective**: distinct levels have distinct wire strings, so
    `Display` never collides — the read/write round-trip is unambiguous. -/
theorem Ladder_toStr_Injective :
    ∀ a b : VLevel, toStr a = toStr b → a = b := by
  intro a b; cases a <;> cases b <;> decide

end Theorems.Macs
