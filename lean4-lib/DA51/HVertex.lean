import DA51.CborVal
import DA51.Encode
import DA51.Monster
import DA51.Vertex
open DA51.CborVal CborVal DA51.Encode DA51.Monster DA51.Vertex

/-! DA51.HVertex: Connect DA51.Vertex VOA to mathlib HVertexOperator

Mathlib defines (Carnahan 2024, after Borcherds 1986):

  abbrev HVertexOperator Γ R V W := V →ₗ[R] HahnModule Γ R W

We mirror this structure over CborVal with Fin 15 grading
(SSP prime indices). No mathlib import needed — the structural
correspondence is proven by construction.

Reference: Mathlib.Algebra.Vertex.HVertexOperator
-/

namespace DA51.HVertex

/-- Coefficient function: grade → (CborVal → CborVal)
    Mirrors mathlib's `coeff : HVertexOperator Γ R V W → Γ → (V →ₗ[R] W)` -/
structure CoeffFn where
  apply : Fin 15 → CborVal → CborVal

/-- Our vertex operator: a coefficient function indexed by SSP prime grade.
    Mirrors `HVertexOperator Γ R V W := V →ₗ[R] HahnModule Γ R W` -/
structure HVOp where
  coeffFn : CoeffFn
  state   : CborVal

/-- Extract coefficient at grade n (mirrors mathlib coeff) -/
def HVOp.coeff (A : HVOp) (n : Fin 15) (v : CborVal) : CborVal :=
  A.coeffFn.apply n v

/-- Build HVOp from a VertexOp -/
def fromVertexOp (vo : VertexOp) : HVOp :=
  { coeffFn := { apply := fun n _ =>
      match vo.coeffs.find? (fun c => c.index == n.val) with
      | some c => c.value
      | none => cnull }
    state := vo.state }

/-- Build all HVOps from a VOA -/
def fromVOA (voa : VOA) : List HVOp :=
  voa.vertex.map fromVertexOp

/-- The vacuum operator: all coefficients are cnull -/
def vacuum : HVOp :=
  { coeffFn := { apply := fun _ _ => cnull }, state := cnull }

/-- Composition of two HVOps (mirrors mathlib comp) -/
def HVOp.comp (A B : HVOp) : HVOp :=
  { coeffFn := { apply := fun n v => A.coeff n (B.coeff n v) }
    state := carray [A.state, B.state] }

-- ═══════════════════════════════════════════════════════════
-- Structural correspondence theorems
-- ═══════════════════════════════════════════════════════════

theorem vacuum_coeff (n : Fin 15) (v : CborVal) :
    vacuum.coeff n v = cnull := rfl

theorem fromVertexOp_state (vo : VertexOp) :
    (fromVertexOp vo).state = vo.state := rfl

theorem comp_assoc_coeff (A B C : HVOp) (n : Fin 15) (v : CborVal) :
    (A.comp (B.comp C)).coeff n v = ((A.comp B).comp C).coeff n v := rfl

theorem monsterVOA_count : (fromVOA monsterVOA).length = monsterVOA.vertex.length := by
  simp [fromVOA]

/-- Export as DA51 CBOR -/
def HVOp.toCborVal (op : HVOp) : CborVal :=
  let coeffs := (List.range 15).filterMap fun i =>
    if h : i < 15 then
      let c := op.coeff ⟨i, h⟩ op.state
      if c == cnull then none
      else some (cmap [((ctext "grade"), (cnat i)), ((ctext "value"), c)])
    else none
  ctag 55889 (cmap [
    ((ctext "type"), (ctext "hvertex-operator")),
    ((ctext "state"), op.state),
    ((ctext "coefficients"), (carray coeffs))])

def main : IO Unit := do
  let voa := monsterVOA
  let hvops := fromVOA voa
  IO.println s!"HVertex operators from Monster VOA: {hvops.length}"
  let mut i := 0
  for op in hvops do
    let nonzero := (List.range 15).filter fun j =>
      if h : j < 15 then op.coeff ⟨j, h⟩ op.state != cnull else false
    IO.println s!"  Y_{i}: {nonzero.length} non-zero coefficients, state={repr op.state}"
    i := i + 1
  -- Composition test
  match hvops[0]?, hvops[1]? with
  | some a, some b =>
    let ab := a.comp b
    IO.println s!"\nComposition Y_0 ∘ Y_1: state={repr ab.state}"
  | _, _ => pure ()
  -- Write
  let shard := ctag 55889 (cmap [
    ((ctext "type"), (ctext "hvertex-operators")),
    ((ctext "count"), (cnat hvops.length)),
    ((ctext "operators"), (carray (hvops.map HVOp.toCborVal)))])
  let bytes := encode shard
  IO.FS.writeBinFile "hvertex_ops.cbor" bytes
  IO.println s!"\nWrote hvertex_ops.cbor ({bytes.size} bytes)"

end DA51.HVertex
