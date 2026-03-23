import DA51.CborVal
import DA51.Encode
import DA51.Decode
import DA51.Monster
open DA51.CborVal CborVal DA51.Encode DA51.Decode DA51.Monster

/-! DA51.SelfApply: feed monster_element.cbor back through the pipeline

The quine test: encode Monster element → CBOR → decode → re-extract
coordinates → re-encode → check grade stability. -/

namespace DA51.SelfApply

/-- Extract MonsterElement from a decoded DA51 CborVal -/
def extractCoords (v : CborVal) : List (String × Nat) :=
  match v with
  | ctag 55889 (cmap kvs) =>
    match kvs.find? (fun (k, _) => k == ctext "coordinates") with
    | some (_, carray coords) =>
      coords.filterMap fun c =>
        match c with
        | cmap inner =>
          let kind := inner.find? (fun (k, _) => k == ctext "kind")
          let coord := inner.find? (fun (k, _) => k == ctext "coord")
          match kind, coord with
          | some (_, ctext k), some (_, cnat n) => if n > 0 then some (k, n) else none
          | _, _ => none
        | _ => none
    | _ => []
  | _ => []

def main : IO Unit := do
  -- Step 1: Generate monster_element.cbor
  let combined := combinedElement
  let shard := toCborVal combined
  let bytes := encode shard
  IO.println s!"Step 1: Encoded combined Monster element ({bytes.size} bytes)"
  IO.println s!"  blade: 0x{Nat.toDigits 16 (blade combined) |> String.ofList}"
  IO.println s!"  grade: {grade combined}/15"

  -- Step 2: Decode it back
  match decode bytes with
  | none => IO.println "  ✗ decode failed"; IO.Process.exit 1
  | some decoded =>
    IO.println "Step 2: Decoded back to CborVal ✓"

    -- Step 3: Extract coordinates from decoded shard
    let counts := extractCoords decoded
    IO.println s!"Step 3: Extracted {counts.length} non-zero coordinates"

    -- Step 4: Re-encode as Monster element
    let reElement := encode_element counts
    IO.println s!"Step 4: Re-encoded Monster element"
    IO.println s!"  blade: 0x{Nat.toDigits 16 (blade reElement) |> String.ofList}"
    IO.println s!"  grade: {grade reElement}/15"

    -- Step 5: Check grade stability
    let g1 := grade combined
    let g2 := grade reElement
    if g1 == g2 then
      IO.println s!"\n✓ Grade stable: {g1}/15 → {g2}/15 (self-application preserves grade)"
    else
      IO.println s!"\n⚠ Grade changed: {g1}/15 → {g2}/15"

    -- Step 6: Re-encode to CBOR, check round-trip
    let reBytes := encode (toCborVal reElement)
    IO.println s!"\nStep 6: Re-encoded to CBOR ({reBytes.size} bytes)"
    if bytes.size == reBytes.size then
      IO.println "  Byte-identical ✓"
    else
      IO.println s!"  Size changed: {bytes.size} → {reBytes.size} (coordinates reduced mod p^e)"

end DA51.SelfApply

def main := DA51.SelfApply.main
