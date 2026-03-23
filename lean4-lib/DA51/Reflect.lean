import Lean
import DA51.CborVal
import DA51.Encode
open DA51.CborVal
open CborVal
open DA51.Encode
open Lean

/-! DA51.Reflect: compile-time introspection of Lean4 environment → DA51 CBOR -/

namespace DA51.Reflect

def constKind : ConstantInfo → String
  | .axiomInfo _  => "axiom"
  | .defnInfo _   => "def"
  | .thmInfo _    => "theorem"
  | .opaqueInfo _ => "opaque"
  | .quotInfo _   => "quot"
  | .inductInfo _ => "inductive"
  | .ctorInfo _   => "ctor"
  | .recInfo _    => "rec"

def constToCborVal (name : Name) (ci : ConstantInfo) : CborVal :=
  cmap [
    ((ctext "subject"), (ctext (toString name))),
    ((ctext "predicate"), (ctext (constKind ci))),
    ((ctext "type"), (ctext (toString ci.type)))
  ]

def reflectEnv (env : Environment) : CborVal :=
  let decls := env.constants.fold (init := (#[] : Array CborVal))
    fun ds name ci =>
      if name.isInternal then ds
      else ds.push (constToCborVal name ci)
  ctag 55889 (cmap [
    ((ctext "source"), (ctext "lean4-environment")),
    ((ctext "decl_count"), (cnat decls.size)),
    ((ctext "decls"), (carray decls.toList))
  ])

elab "#da51_reflect" : command => do
  let env ← Lean.Elab.Command.liftCoreM Lean.MonadEnv.getEnv
  let shard := reflectEnv env
  let bytes := encode shard
  Lean.Elab.Command.liftTermElabM do
    Lean.logInfo m!"DA51 reflect: {bytes.size} bytes CBOR"

elab "#da51_reflect_to" path:str : command => do
  let env ← Lean.Elab.Command.liftCoreM Lean.MonadEnv.getEnv
  let shard := reflectEnv env
  let bytes := encode shard
  let p := path.getString
  IO.FS.writeBinFile p bytes
  Lean.Elab.Command.liftTermElabM do
    Lean.logInfo m!"DA51 reflect: wrote {bytes.size} bytes to {p}"

end DA51.Reflect
