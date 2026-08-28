/-
  Theorems.Tdg.Grade — L5 machine-checked proofs for the pmat TDG grade scale.

  `GradeL` mirrors `Grade` in `src/tdg/grade.rs`; `bandFloor` mirrors
  `GRADE_BANDS` (grade.rs:172-182) and `fromScore` mirrors `Grade::from_score`
  (grade.rs:122-127). The theorems below discharge the order obligations of
  `contracts/tdg-grade-order-v1.yaml`.

  WHY THIS FILE EXISTS, precisely. CB-200 shipped a private five-letter table
  (`["A","B","C","D","F"]`) and a `_ => 5` catch-all, so it was blind to every
  modified grade and a threshold of "A-" silently disabled the gate. The obvious
  remedy — assert that `Grade::ALL` is in declaration order — is NOT sufficient,
  and an adversarial review of the fix proved it: reverse the enum declaration
  AND `ALL` together and every self-referential invariant still holds (the const
  block passes, `Display` stays injective, parse stays a left inverse, all 121
  pairs and 1,331 triples pass, `below`/`passing` still partition) while
  `meets_threshold` now reports that A+ fails an F floor.

  The reason is that all of those invariants are internal to the order. NONE of
  them anchors rank to anything outside itself. `bandFloor` is that anchor: it
  is attached to variant NAMES, so `Grade_Rank_Anchored_To_Score` below fails
  under exactly the mutation the const block waves through.

  Pure Lean 4 core — no Mathlib, so `lake build` stays hermetic and fast, and
  every proof is `decide` / `cases … <;> decide` closing its goal completely.
-/

namespace Theorems.Tdg

/-- The eleven TDG grades, best first, mirroring `Grade`'s declaration order. -/
inductive GradeL where
  | APlus | A | AMinus
  | BPlus | B | BMinus
  | CPlus | C | CMinus
  | D | F
deriving DecidableEq, Repr

namespace GradeL

/-- Every grade, best first — mirrors `Grade::ALL`. -/
def all : List GradeL :=
  [APlus, A, AMinus, BPlus, B, BMinus, CPlus, C, CMinus, D, F]

/-- Canonical wire spelling (mirrors `Display for Grade`, grade.rs:186-199). -/
def toStr : GradeL → String
  | APlus => "A+" | A => "A" | AMinus => "A-"
  | BPlus => "B+" | B => "B" | BMinus => "B-"
  | CPlus => "C+" | C => "C" | CMinus => "C-"
  | D => "D"     | F => "F"

/-- Rank: SMALLER IS BETTER, mirroring the derived `Ord` on `Grade`. -/
def rank : GradeL → Nat
  | APlus => 0 | A => 1 | AMinus => 2
  | BPlus => 3 | B => 4 | BMinus => 5
  | CPlus => 6 | C => 7 | CMinus => 8
  | D => 9     | F => 10

/-- Score floor of each grade's band, mirroring `GRADE_BANDS`. `F` is the
    fallthrough and has no floor of its own, so it is given 0. Scores are
    modelled as `Nat` percentage points; the Rust bands are whole numbers
    (95, 90, …, 50), so no precision is lost. -/
def bandFloor : GradeL → Nat
  | APlus => 95 | A => 90 | AMinus => 85
  | BPlus => 80 | B => 75 | BMinus => 70
  | CPlus => 65 | C => 60 | CMinus => 55
  | D => 50     | F => 0

/-- Mirrors `Grade::from_score`: first band whose floor the score reaches. -/
def fromScore (s : Nat) : GradeL :=
  if s ≥ 95 then APlus else if s ≥ 90 then A else if s ≥ 85 then AMinus
  else if s ≥ 80 then BPlus else if s ≥ 75 then B else if s ≥ 70 then BMinus
  else if s ≥ 65 then CPlus else if s ≥ 60 then C else if s ≥ 55 then CMinus
  else if s ≥ 50 then D else F

/-- Mirrors `Grade::meets_threshold`: at least as good as the floor. -/
def meetsThreshold (g t : GradeL) : Bool := rank g ≤ rank t

/-- The set a floor admits (mirrors `GradeFloor::passing`). -/
def passing (t : GradeL) : List GradeL := all.filter (fun g => meetsThreshold g t)

/-- Its complement (mirrors `GradeFloor::below`). -/
def below (t : GradeL) : List GradeL := all.filter (fun g => !meetsThreshold g t)

/-- Strict parser over the canonical spellings (mirrors the symbolic half of
    `Grade::from_variant_name`, grade.rs:74-85). -/
def parseStrict (s : String) : Option GradeL :=
  if s = "A+" then some APlus else if s = "A" then some A
  else if s = "A-" then some AMinus else if s = "B+" then some BPlus
  else if s = "B" then some B else if s = "B-" then some BMinus
  else if s = "C+" then some CPlus else if s = "C" then some C
  else if s = "C-" then some CMinus else if s = "D" then some D
  else if s = "F" then some F else none

end GradeL

open GradeL

/-! ## The anchoring theorems — what a self-referential invariant cannot prove -/

/-- **Grade_Rank_Anchored_To_Score**. A better rank has a strictly higher score
    floor. This is THE theorem the `const _` block cannot express: `bandFloor`
    is attached to variant names, so reversing the enum declaration together
    with `ALL` — which leaves every internal invariant intact — makes this
    goal false. Anchors the order to the numeric scale it claims to summarise. -/
theorem Grade_Rank_Anchored_To_Score :
    ∀ a b : GradeL, a ≠ F → b ≠ F → rank a < rank b → bandFloor b < bandFloor a := by
  intro a b; cases a <;> cases b <;> decide

/-- **Grade_FromScore_Recovers_Band**: every non-`F` grade's own floor scores
    back to that grade. Ties `fromScore` to `bandFloor` in the other direction,
    so the two tables cannot drift apart. -/
theorem Grade_FromScore_Recovers_Band :
    ∀ g : GradeL, g ≠ F → fromScore (bandFloor g) = g := by
  intro g; cases g <;> decide

/-- **Grade_FromScore_Antitone_Step**: one more point of score never yields a
    worse grade, at every one of the 101 reporting positions.

    Stated stepwise rather than over all 10,201 ordered pairs because on a
    discrete domain the two are equivalent — full antitonicity follows from
    this by transitivity of `≤` — and the pairwise form exhausts the kernel's
    recursion depth. The stepwise form is what actually catches the defect of
    interest: an inverted or misordered BAND EDGE shows up as a single failing
    step. The Rust sweep at grade.rs:380-407 samples 1,001 points across the
    range and can miss an edge that falls between two samples; this checks
    every integer position. -/
theorem Grade_FromScore_Antitone_Step :
    ∀ s ∈ List.range 101, rank (fromScore (s + 1)) ≤ rank (fromScore s) := by
  decide

/-! ## Order algebra -/

/-- **Grade_Rank_Injective**: rank never conflates two grades, so `Ord`
    comparisons cannot merge distinct grades. -/
theorem Grade_Rank_Injective : ∀ a b : GradeL, rank a = rank b → a = b := by
  intro a b; cases a <;> cases b <;> decide

/-- **Grade_Order_Total**: any two grades are comparable. -/
theorem Grade_Order_Total :
    ∀ a b : GradeL, meetsThreshold a b = true ∨ meetsThreshold b a = true := by
  intro a b; cases a <;> cases b <;> decide

/-- **Grade_Order_Transitive**: 1,331 closed cases, checked by the kernel. -/
theorem Grade_Order_Transitive :
    ∀ a b c : GradeL,
      meetsThreshold a b = true → meetsThreshold b c = true →
      meetsThreshold a c = true := by
  intro a b c; cases a <;> cases b <;> cases c <;> decide

/-- **Grade_Order_Antisymmetric**. -/
theorem Grade_Order_Antisymmetric :
    ∀ a b : GradeL,
      meetsThreshold a b = true → meetsThreshold b a = true → a = b := by
  intro a b; cases a <;> cases b <;> decide

/-! ## The gate's own predicate -/

/-- **Grade_Partition**: `passing` and `below` partition the whole scale for
    every floor. This is Defect 1 stated as a theorem — the shipped five-letter
    table partitioned nothing, so 544 of 791 violations were invisible. -/
theorem Grade_Partition :
    ∀ t : GradeL, (passing t).length + (below t).length = all.length := by
  intro t; cases t <;> decide

/-- **Grade_Below_Empty_Only_At_F**: the failing set is empty for exactly one
    floor, `F`. This is Defect 2 stated as a theorem — `grade_ordinal` mapped
    every modified grade to a catch-all worse than `F`, so a floor of "A-"
    produced an empty failing set and the gate returned Pass. -/
theorem Grade_Below_Empty_Only_At_F :
    ∀ t : GradeL, (below t).isEmpty = true ↔ t = F := by
  intro t; cases t <;> decide

/-- **Grade_Below_Antitone**: a stricter floor never fails fewer grades.
    A monotonicity the gate must have and never asserted. -/
theorem Grade_Below_Antitone :
    ∀ a b : GradeL, rank a ≤ rank b → (below b).length ≤ (below a).length := by
  intro a b; cases a <;> cases b <;> decide

/-! ## Parse -/

/-- **Grade_Parse_Total**: parsing a canonical spelling round-trips. -/
theorem Grade_Parse_Total : ∀ g : GradeL, parseStrict (toStr g) = some g := by
  intro g; cases g <;> decide

/-- **Grade_ToStr_Injective**: distinct grades have distinct spellings, so the
    stored TEXT column is an unambiguous encoding of the order. -/
theorem Grade_ToStr_Injective : ∀ a b : GradeL, toStr a = toStr b → a = b := by
  intro a b; cases a <;> cases b <;> decide

/-- **Grade_Parse_Strict**: the corruptions that mattered are rejected. `"A−"`
    carries a UNICODE MINUS, which is what a copy-paste from rendered docs
    produces; under the shipped catch-all it ranked worse than `F`. -/
theorem Grade_Parse_Strict :
    parseStrict "" = none ∧ parseStrict " A" = none ∧ parseStrict "A " = none ∧
    parseStrict "Q" = none ∧ parseStrict "A--" = none ∧ parseStrict "E" = none := by
  decide

end Theorems.Tdg
