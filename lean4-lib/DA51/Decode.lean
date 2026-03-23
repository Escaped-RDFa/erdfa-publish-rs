import DA51.CborVal
import DA51.Encode
open DA51.CborVal CborVal DA51.Encode

/-! DA51.Decode: CBOR ByteArray → Option CborVal

Minimal decoder for the CBOR subset used by DA51.Encode.
Round-trip verified via #eval tests. -/

namespace DA51.Decode

def readHead (bs : ByteArray) (pos : Nat) : Option (Nat × Nat × Nat) :=
  if pos < bs.size then
    let b := bs.data[pos]!
    let major := (b >>> 5).toNat
    let info := (b &&& 0x1f).toNat
    if info < 24 then some (major, info, pos + 1)
    else if info == 24 && pos + 1 < bs.size then
      some (major, bs.data[pos+1]!.toNat, pos + 2)
    else if info == 25 && pos + 2 < bs.size then
      some (major, bs.data[pos+1]!.toNat * 256 + bs.data[pos+2]!.toNat, pos + 3)
    else if info == 26 && pos + 4 < bs.size then
      some (major,
        bs.data[pos+1]!.toNat * 0x1000000 + bs.data[pos+2]!.toNat * 0x10000 +
        bs.data[pos+3]!.toNat * 0x100 + bs.data[pos+4]!.toNat, pos + 5)
    else if info == 27 && pos + 8 < bs.size then
      some (major,
        bs.data[pos+1]!.toNat * 0x100000000000000 + bs.data[pos+2]!.toNat * 0x1000000000000 +
        bs.data[pos+3]!.toNat * 0x10000000000 + bs.data[pos+4]!.toNat * 0x100000000 +
        bs.data[pos+5]!.toNat * 0x1000000 + bs.data[pos+6]!.toNat * 0x10000 +
        bs.data[pos+7]!.toNat * 0x100 + bs.data[pos+8]!.toNat, pos + 9)
    else none
  else none

partial def decodeAt (bs : ByteArray) (pos : Nat) : Option (CborVal × Nat) := do
  let (major, val, pos') ← readHead bs pos
  match major with
  | 0 => some (cnat val, pos')
  | 1 => some (cneg (val + 1), pos')
  | 2 =>
    if pos' + val ≤ bs.size then
      let bytes := (List.range val).map fun i => bs.data[pos' + i]!.toNat
      some (cbytes bytes, pos' + val)
    else none
  | 3 =>
    if pos' + val ≤ bs.size then
      some (ctext (String.fromUTF8! (bs.extract pos' (pos' + val))), pos' + val)
    else none
  | 4 =>
    let mut items : List CborVal := []
    let mut p := pos'
    for _ in List.range val do
      let (item, p') ← decodeAt bs p
      items := items ++ [item]
      p := p'
    some (carray items, p)
  | 5 =>
    let mut kvs : List (CborVal × CborVal) := []
    let mut p := pos'
    for _ in List.range val do
      let (k, p1) ← decodeAt bs p
      let (v, p2) ← decodeAt bs p1
      kvs := kvs ++ [(k, v)]
      p := p2
    some (cmap kvs, p)
  | 6 =>
    let (inner, p'') ← decodeAt bs pos'
    some (ctag val inner, p'')
  | 7 =>
    if val == 20 then some (cbool false, pos')
    else if val == 21 then some (cbool true, pos')
    else if val == 22 then some (cnull, pos')
    else some (cfloat val, pos')
  | _ => none

def decode (bs : ByteArray) : Option CborVal :=
  match decodeAt bs 0 with
  | some (v, _) => some v
  | none => none

-- ═══════════════════════════════════════════════════════════
-- Round-trip: decode ∘ encode = some (verified by #eval)
-- ═══════════════════════════════════════════════════════════

private def check (name : String) (v : CborVal) : IO Unit := do
  let encoded := encode v
  match decode encoded with
  | some v' =>
    if toString (repr v) == toString (repr v') then
      IO.println s!"  ✓ {name}"
    else
      IO.println s!"  ✗ {name}: got {repr v'}"
      IO.Process.exit 1
  | none =>
    IO.println s!"  ✗ {name}: decode failed"
    IO.Process.exit 1

def main : IO Unit := do
  IO.println "decode ∘ encode round-trip tests:"
  check "null" cnull
  check "true" (cbool true)
  check "false" (cbool false)
  check "nat 0" (cnat 0)
  check "nat 23" (cnat 23)
  check "nat 24" (cnat 24)
  check "nat 255" (cnat 255)
  check "nat 256" (cnat 256)
  check "nat 65535" (cnat 65535)
  check "nat 65536" (cnat 65536)
  check "neg 1" (cneg 1)
  check "text" (ctext "hello")
  check "text utf8" (ctext "λ∀∃")
  check "empty array" (carray [])
  check "array" (carray [cnat 1, cnat 2, cnat 3])
  check "empty map" (cmap [])
  check "map" (cmap [((ctext "a"), (cnat 1))])
  check "tag" (ctag 55889 (ctext "test"))
  check "bytes" (cbytes [0, 127, 255])
  -- The key test: DA51 RDF triple shard
  check "DA51 shard" (ctag 55889 (cmap [
    ((ctext "subject"), (ctext "main")),
    ((ctext "predicate"), (ctext "fn")),
    ((ctext "object"), (ctext "main.rs")),
    ((ctext "prime"), (cnat 2)),
    ((ctext "blade"), (cnat 1))
  ]))
  -- Nested: array of maps in a tag
  check "nested shard" (ctag 55889 (cmap [
    ((ctext "decls"), (carray [
      (cmap [((ctext "s"), (ctext "DIM")), ((ctext "p"), (ctext "const"))]),
      (cmap [((ctext "s"), (ctext "main")), ((ctext "p"), (ctext "fn"))])
    ]))
  ]))
  IO.println "All round-trip tests pass ✓"

end DA51.Decode
