import DA51.CborVal
import DA51.Encode
import DA51.Decode
open DA51.CborVal CborVal DA51.Encode DA51.Decode

/-! DA51.Exec: Read DA51 CBOR erdfa file, execute it.

A DA51 shard is not just data — it's a program.
Each RDF triple (subject, predicate, object) is an instruction:
  predicate = opcode
  subject   = operand
  object    = context

Execution: walk the triples, dispatch on predicate, accumulate state.
-/

namespace DA51.Exec

/-- Execution state -/
structure State where
  env    : List (String × CborVal)  -- bindings
  output : List CborVal             -- emitted values
  pc     : Nat                      -- program counter
deriving Repr

def State.empty : State := ⟨[], [], 0⟩

def State.bind (s : State) (name : String) (val : CborVal) : State :=
  { s with env := (name, val) :: s.env }

def State.emit (s : State) (val : CborVal) : State :=
  { s with output := s.output ++ [val] }

def State.lookup (s : State) (name : String) : Option CborVal :=
  s.env.find? (fun (k, _) => k == name) |>.map (·.2)

/-- Execute one triple -/
def execTriple (s : State) (subj pred obj : String) : State :=
  let s := { s with pc := s.pc + 1 }
  match pred with
  | "fn"     => s.bind subj (ctext s!"fn:{subj}@{obj}")
  | "const"  => s.bind subj (ctext s!"const:{subj}")
  | "struct" => s.bind subj (cmap [((ctext "type"), (ctext "struct")), ((ctext "name"), (ctext subj))])
  | "enum"   => s.bind subj (cmap [((ctext "type"), (ctext "enum")), ((ctext "name"), (ctext subj))])
  | "impl"   => s.emit (ctext s!"impl {subj} for {obj}")
  | "trait"  => s.bind subj (cmap [((ctext "type"), (ctext "trait")), ((ctext "name"), (ctext subj))])
  | "use"    => s.emit (ctext s!"import {subj}")
  | "mod"    => s.bind subj (ctext s!"module:{subj}")
  | "type"   => s.bind subj (ctext s!"type:{subj}")
  | "let"    => s.bind subj (ctext s!"let:{subj}")
  | "field"  => s.emit (ctext s!"field {subj} in {obj}")
  | "variant"=> s.emit (ctext s!"variant {subj} of {obj}")
  | "method" => s.emit (ctext s!"method {subj} on {obj}")
  | "macro"  => s.bind subj (ctext s!"macro:{subj}")
  | "static" => s.bind subj (ctext s!"static:{subj}")
  | _        => s.emit (ctext s!"unknown:{pred} {subj}")

/-- Extract and execute all triples from a DA51 shard -/
def exec (shard : CborVal) : State :=
  let triples : List (String × String × String) := match shard with
    | ctag 55889 (cmap kvs) =>
      match kvs.find? (fun (k, _) => k == ctext "decls") with
      | some (_, carray decls) => decls.filterMap fun d =>
        match d with
        | cmap inner =>
          let s := inner.find? (fun (k, _) => k == ctext "subject")
          let p := inner.find? (fun (k, _) => k == ctext "predicate")
          let o := inner.find? (fun (k, _) => k == ctext "object")
          match s, p, o with
          | some (_, ctext subj), some (_, ctext pred), some (_, ctext obj) =>
            some (subj, pred, obj)
          | _, _, _ => none
        | _ => none
      | _ => []
    | _ => []
  triples.foldl (fun s (subj, pred, obj) => execTriple s subj pred obj) State.empty

/-- Package execution result as DA51 shard -/
def resultShard (input : String) (st : State) : CborVal :=
  ctag 55889 (cmap [
    ((ctext "type"), (ctext "exec-result")),
    ((ctext "input"), (ctext input)),
    ((ctext "instructions"), (cnat st.pc)),
    ((ctext "bindings"), (cnat st.env.length)),
    ((ctext "emissions"), (cnat st.output.length)),
    ((ctext "env"), (cmap (st.env.map fun (k, v) => ((ctext k), v)))),
    ((ctext "output"), (carray st.output))
  ])

def main (args : List String) : IO Unit := do
  let path := args.getLast? |>.getD "../shards/binding-ontology/clifford_rs.cbor"
  IO.println s!"DA51 exec: {path}"
  let bytes ← IO.FS.readBinFile path
  match decode bytes with
  | none => IO.println "  ✗ decode failed"; IO.Process.exit 1
  | some shard =>
    let st := exec shard
    IO.println s!"  instructions: {st.pc}"
    IO.println s!"  bindings: {st.env.length}"
    IO.println s!"  emissions: {st.output.length}"
    IO.println "\n  env:"
    for (k, _) in st.env.take 10 do
      IO.println s!"    {k}"
    if st.env.length > 10 then IO.println s!"    ... ({st.env.length - 10} more)"
    IO.println "\n  output:"
    for v in st.output.take 10 do
      match v with
      | ctext t => IO.println s!"    {t}"
      | _ => IO.println s!"    {repr v}"
    if st.output.length > 10 then IO.println s!"    ... ({st.output.length - 10} more)"
    -- Write result
    let result := resultShard path st
    let outBytes := encode result
    let outPath := path ++ ".exec.cbor"
    IO.FS.writeBinFile outPath outBytes
    IO.println s!"\n  wrote {outPath} ({outBytes.size} bytes)"

end DA51.Exec

def main := DA51.Exec.main
