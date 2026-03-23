namespace DA51.CborVal

inductive CborVal where
  | cnat    : Nat → CborVal
  | cneg    : Nat → CborVal
  | cbytes  : List Nat → CborVal
  | ctext   : String → CborVal
  | carray  : List CborVal → CborVal
  | cmap    : List (CborVal × CborVal) → CborVal
  | ctag    : Nat → CborVal → CborVal
  | cbool   : Bool → CborVal
  | cnull   : CborVal
  | cfloat  : Nat → CborVal
deriving Repr, BEq

def da51Tag : Nat := 55889

end DA51.CborVal
