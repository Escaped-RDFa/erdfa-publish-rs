import DA51.CborVal
import DA51.Encode
import DA51.Monster
open DA51.CborVal CborVal DA51.Encode DA51.Monster

/-! DA51.Hecke: Hecke operator τ(p) walk on Monster element coordinates -/

namespace DA51.Hecke

def heckeStep (me : MonsterElement) (p : Nat) : MonsterElement :=
  let coords := slots.map fun s =>
    match me.coords.find? (fun (k, _, _) => k == s.kind) with
    | some (k, pr, c) => (k, pr, (c + p) % pow pr s.exp)
    | none => (s.kind, s.prime, p % pow s.prime s.exp)
  ⟨coords⟩

def iterate (me : MonsterElement) (p : Nat) : Nat → MonsterElement
  | 0 => me
  | n + 1 => heckeStep (iterate me p n) p

def walkGrades (me : MonsterElement) (p steps : Nat) : List Nat :=
  (List.range (steps + 1)).map fun i => grade (iterate me p i)

def gradeStable (grades : List Nat) : Bool :=
  match grades with
  | [] => true
  | g :: rest => rest.all (· == g)

def main : IO Unit := do
  let me := combinedElement
  IO.println s!"Initial: blade 0x{Nat.toDigits 16 (blade me) |> String.ofList}, grade {grade me}/15"

  let primes := [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]
  for p in primes do
    let grades := walkGrades me p 10
    let stable := gradeStable grades
    let minG := grades.foldl min 15
    let maxG := grades.foldl max 0
    IO.println s!"  τ({p}) × 10: grade [{minG},{maxG}]/15 {if stable then "STABLE" else "VARIES"}"

  IO.println "\nτ(2) walk, 100 steps:"
  let g100 := walkGrades me 2 100
  let minG := g100.foldl min 15
  let maxG := g100.foldl max 0
  IO.println s!"  grade [{minG},{maxG}]/15, stable: {gradeStable g100}"

  -- Check blade evolution
  IO.println "\nBlade evolution (first 5 steps of τ(2)):"
  for i in List.range 6 do
    let e := iterate me 2 i
    IO.println s!"  step {i}: blade 0x{Nat.toDigits 16 (blade e) |> String.ofList} grade {grade e}/15 fp={crt_fingerprint e}"

end DA51.Hecke

def main := DA51.Hecke.main
