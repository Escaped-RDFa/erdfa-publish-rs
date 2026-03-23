import DA51.CborVal
import DA51.Encode
open DA51.CborVal CborVal DA51.Encode

/-! DA51.Ternary: RDF triples encoded in 3^20 (Monster 3-Sylow)

The Monster group order has 3^20 as its 3-part.
RDF triples are ternary: (subject, predicate, object).
We encode triples as points in ℤ/3^20, the natural
ternary address space from the Monster's 3-Sylow subgroup.

Partition of 20 ternary digits:
  digits  0-6  (3^7 = 2187) → subject
  digits  7-13 (3^7 = 2187) → predicate
  digits 14-19 (3^6 =  729) → object
-/

namespace DA51.Ternary

def pow3 : Nat → Nat
  | 0 => 1
  | n + 1 => 3 * pow3 n

-- 3^20 = Monster 3-Sylow order
def monster3 : Nat := pow3 20

theorem monster3_val : monster3 = 3486784401 := by native_decide

-- Partition: 7 + 7 + 6 = 20 ternary digits
def subjBits : Nat := 7   -- 3^7 = 2187
def predBits : Nat := 7
def objBits  : Nat := 6   -- 3^6 = 729

theorem partition_sum : subjBits + predBits + objBits = 20 := by native_decide

-- Hash a string to a ternary range [0, 3^n)
def ternaryHash (s : String) (n : Nat) : Nat :=
  let h := s.foldl (fun acc c => acc * 31 + c.toNat) 5381
  h % pow3 n

-- Encode an RDF triple as a point in [0, 3^20)
def encodeTriple (subj pred obj : String) : Nat :=
  let s := ternaryHash subj subjBits
  let p := ternaryHash pred predBits
  let o := ternaryHash obj objBits
  s + p * pow3 subjBits + o * pow3 (subjBits + predBits)

-- Decode back to (subject_hash, predicate_hash, object_hash)
def decodeTriple (n : Nat) : Nat × Nat × Nat :=
  let s := n % pow3 subjBits
  let p := (n / pow3 subjBits) % pow3 predBits
  let o := n / pow3 (subjBits + predBits)
  (s, p, o)

-- Round-trip on concrete examples
theorem roundtrip_main :
    decodeTriple (encodeTriple "main" "fn" "main.rs") =
    (ternaryHash "main" subjBits, ternaryHash "fn" predBits, ternaryHash "main.rs" objBits) := by
  native_decide

theorem roundtrip_dim :
    decodeTriple (encodeTriple "DIM" "const" "clifford.rs") =
    (ternaryHash "DIM" subjBits, ternaryHash "const" predBits, ternaryHash "clifford.rs" objBits) := by
  native_decide

-- The encoded value fits in 3^20
theorem encode_main_in_range :
    encodeTriple "main" "fn" "main.rs" < monster3 := by native_decide

theorem encode_dim_in_range :
    encodeTriple "DIM" "const" "clifford.rs" < monster3 := by native_decide

-- Ternary digits of a number
def ternaryDigits : Nat → Nat → List Nat
  | _, 0 => []
  | n, k + 1 => (n % 3) :: ternaryDigits (n / 3) k

-- Convert a DA51 decl triple to its 3^20 encoding
def declToTernary (subj pred obj : String) : CborVal :=
  let t := encodeTriple subj pred obj
  cmap [
    ((ctext "subject"), (ctext subj)),
    ((ctext "predicate"), (ctext pred)),
    ((ctext "object"), (ctext obj)),
    ((ctext "ternary_addr"), (cnat t)),
    ((ctext "ternary_digits"), (carray (ternaryDigits t 20 |>.map (fun d => cnat d))))
  ]

-- Concrete example from our pipeline
def example_main : CborVal := declToTernary "main" "fn" "main.rs"
def example_const : CborVal := declToTernary "DIM" "const" "clifford.rs"

-- Wrap in DA51
def ternaryOntology (triples : List (String × String × String)) : CborVal :=
  ctag 55889 (cmap [
    ((ctext "encoding"), (ctext "monster-3-sylow")),
    ((ctext "base"), (cnat 3)),
    ((ctext "exponent"), (cnat 20)),
    ((ctext "capacity"), (cnat monster3)),
    ((ctext "triples"), (carray (triples.map fun (s, p, o) => declToTernary s p o)))
  ])

def main : IO Unit := do
  let triples := [
    ("main", "fn", "main.rs"),
    ("DIM", "const", "clifford.rs"),
    ("serde::{..}", "use", "clifford.rs"),
    ("CliffordElement", "struct", "clifford.rs"),
    ("MonsterGroup", "enum", "monster.rs")
  ]
  let shard := ternaryOntology triples
  let bytes := encode shard
  IO.FS.writeBinFile "ternary_ontology.cbor" bytes
  IO.println s!"Wrote {bytes.size} bytes"
  for (s, p, o) in triples do
    let t := encodeTriple s p o
    let digits := ternaryDigits t 20
    IO.println s!"  ({s}, {p}, {o}) → 3^20 addr {t} digits {digits}"

end DA51.Ternary
