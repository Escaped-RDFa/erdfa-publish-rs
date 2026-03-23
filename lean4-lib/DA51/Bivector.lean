import DA51.CborVal
import DA51.Encode
open DA51.CborVal CborVal DA51.Encode

/-! DA51.Bivector: Break 1D locality, trivector census

The reviewer noted that 416/455 trivectors come from nearest-neighbor
chain topology. We add long-range bivectors and check:
1. Do the missing 39 trivectors appear?
2. Does (47,59,71) = (e12,e13,e14) become special?
-/

namespace DA51.Bivector

-- SSP primes indexed 0..14
def primes : Array Nat := #[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]

-- A bivector is a pair (i, j) with i < j
-- Adjacent: |i - j| = 1 (nearest-neighbor chain)
-- Long-range: |i - j| > 1

def allBivectors : List (Nat × Nat) :=
  (List.range 15).flatMap fun i =>
    (List.range 15).filterMap fun j =>
      if i < j then some (i, j) else none

def adjacentBivectors : List (Nat × Nat) :=
  allBivectors.filter fun (i, j) => j - i == 1

def longRangeBivectors : List (Nat × Nat) :=
  allBivectors.filter fun (i, j) => j - i > 1

-- A trivector is a triple (i, j, k) with i < j < k
def allTrivectors : List (Nat × Nat × Nat) :=
  (List.range 15).flatMap fun i =>
    (List.range 15).flatMap fun j =>
      (List.range 15).filterMap fun k =>
        if i < j && j < k then some (i, j, k) else none

-- Generate trivectors reachable from a seed grade-1 element
-- via wedge product with bivectors
-- A trivector (i,j,k) is reachable if there exist bivectors
-- (a,b) and index c such that {a,b,c} = {i,j,k}
def reachableFrom (bivectors : List (Nat × Nat)) : List (Nat × Nat × Nat) :=
  let trivs := bivectors.flatMap fun (a, b) =>
    (List.range 15).filterMap fun c =>
      if c != a && c != b then
        let triple := [a, b, c]
        let sorted := triple.mergeSort (· ≤ ·)
        match sorted with
        | [i, j, k] => some (i, j, k)
        | _ => none
      else none
  -- deduplicate
  trivs.foldl (fun acc t => if acc.contains t then acc else acc ++ [t]) []

-- The moonshine trivector
def moonshine : Nat × Nat × Nat := (12, 13, 14)  -- (47, 59, 71)

def main : IO Unit := do
  let total := allTrivectors.length
  IO.println s!"Total trivectors: C(15,3) = {total}"
  IO.println s!"Total bivectors: C(15,2) = {allBivectors.length}"
  IO.println s!"  Adjacent (|i-j|=1): {adjacentBivectors.length}"
  IO.println s!"  Long-range (|i-j|>1): {longRangeBivectors.length}"

  -- 1. Adjacent-only (1D chain)
  let adjReach := reachableFrom adjacentBivectors
  IO.println s!"\n1D chain (adjacent only): {adjReach.length}/{total} trivectors"
  let missing1D := allTrivectors.filter fun t => !adjReach.contains t
  IO.println s!"  Missing: {missing1D.length}"
  IO.println s!"  Moonshine (12,13,14) reachable: {adjReach.contains moonshine}"

  -- 2. All bivectors (break locality)
  let allReach := reachableFrom allBivectors
  IO.println s!"\nFull connectivity (all bivectors): {allReach.length}/{total} trivectors"
  let missingFull := allTrivectors.filter fun t => !allReach.contains t
  IO.println s!"  Missing: {missingFull.length}"
  IO.println s!"  Moonshine (12,13,14) reachable: {allReach.contains moonshine}"

  -- 3. Check which missing 1D trivectors contain e0 or e1
  let missingWithE0 := missing1D.filter fun (i, _, _) => i == 0
  let missingWithE1 := missing1D.filter fun (i, j, _) => i == 1 || j == 1
  IO.println s!"\nMissing 1D trivectors containing e0: {missingWithE0.length}"
  IO.println s!"Missing 1D trivectors containing e1: {missingWithE1.length}"

  -- 4. Add specific long-range bivectors and check incrementally
  IO.println "\nIncremental long-range addition:"
  let ranges := [(2, "skip-1"), (3, "skip-2"), (5, "skip-4"), (7, "skip-6"), (14, "full")]
  for (maxSkip, label) in ranges do
    let bvs := allBivectors.filter fun (i, j) => j - i ≤ maxSkip
    let reach := reachableFrom bvs
    let hasMoon := reach.contains moonshine
    IO.println s!"  skip≤{maxSkip} ({label}): {reach.length}/{total} trivectors, moonshine: {hasMoon}"

  -- 5. Strength analysis: how many generation paths reach each trivector?
  IO.println "\nTrivector generation multiplicity (top 10 and moonshine):"
  let counts := allTrivectors.map fun t =>
    let paths := allBivectors.foldl (fun acc (a, b) =>
      let c_options := (List.range 15).filter fun c =>
        c != a && c != b &&
        (let s := [a,b,c].mergeSort (· ≤ ·); match s with | [i,j,k] => (i,j,k) == t | _ => false)
      acc + c_options.length) 0
    (t, paths)
  let sorted := counts.mergeSort (fun a b => a.2 > b.2)
  for (t, n) in sorted.take 10 do
    let (i, j, k) := t
    IO.println s!"  ({primes[i]!},{primes[j]!},{primes[k]!}) = e{i}∧e{j}∧e{k}: {n} paths"
  -- moonshine specifically
  match counts.find? (fun (t, _) => t == moonshine) with
  | some (_, n) => IO.println s!"  moonshine (47,59,71) = e12∧e13∧e14: {n} paths"
  | none => IO.println "  moonshine not found"

end DA51.Bivector

def main := DA51.Bivector.main
