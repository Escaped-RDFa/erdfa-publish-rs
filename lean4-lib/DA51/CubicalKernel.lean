import Mathlib.Data.Nat.Prime
import Mathlib.Algebra.Group.Defs
import Mathlib.Tactic

namespace Borcherds.Cubical

/-- The 15 supersingular primes, indexed 0..14 for Cl(15,0,0) basis. -/
def sspPrimes : List Nat := [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]

theorem sspPrimes_length : sspPrimes.length = 15 := by native_decide

/-- Index of a prime in the SSP list (for blade basis vector). -/
def sspIndex (p : Nat) : Option Nat :=
  sspPrimes.indexOf? p

/-- Conformal weight h = log(n) / log(P) where P = 71. -/
def conformalWeight (n : Nat) : Float :=
  if n = 0 then 0.0 else (Float.log n.toFloat) / (Float.log 71.0)

/-- Ramanujan tau values for the 15 SSPs (precomputed). -/
def ramanujanTau : Array Int := #[
  -24, 252, 4830, -16744, 534612, -577738, -6905934, 10661420,
  18643272, -73279080, 24647814, -349561482, -2115101568,
  6066552204, -1437329712
]

/-- Blade in Cl(15,0,0): represented by a bitmask (grade = popcount). -/
structure Blade where
  mask : Nat
deriving Repr, DecidableEq

def Blade.grade (b : Blade) : Nat := Nat.popCount b.mask

/-- SSP-pure predicate: all prime factors are in sspPrimes. -/
def IsSSPPure (n : Nat) : Prop :=
  n = 1 ∨ ∀ p, Nat.Prime p → p ∣ n → p ∈ sspPrimes

/-- 0-cell: a point in the complex (prime token or 1). -/
inductive Cell0 : Type
  | prime (p : Nat) (h : p ∈ sspPrimes)
  | unit
deriving Repr

/-- 1-cell: a divisibility path p^e ∣ n. -/
structure Cell1 where
  source : Cell0
  target : Nat
  exponent : Nat
  blade : Blade
  weight : Float
deriving Repr

/-- 2-cell: a square relating two paths. -/
inductive Cell2 : Type
  | add (a b c : Nat) (h : a = b + c)
  | decomp (n : Nat) (factors : List Nat) (h : n = factors.prod)
deriving Repr

/-- Lift an integer to its Clifford blade (product of SSP factors). -/
def liftToBlade (n : Nat) : Blade :=
  let go (m : Nat) (p : Nat) (idx : Nat) : Nat :=
    if p ∣ n then m ||| (1 <<< idx) else m
  let mask := sspPrimes.enum.foldl (init := 0) fun m (idx, p) => go m p idx
  ⟨mask⟩

/-- The full cubical complex. -/
structure CubicalComplex where
  zeroCells : List Cell0
  oneCells  : List Cell1
  twoCells  : List Cell2

-- ═══ Verified arithmetic (1-cells) ═══

theorem crown_product : 47 * 59 * 71 = 196883 := by native_decide
theorem griess_plus_one : 196884 = 196883 + 1 := by native_decide
theorem second_rep_decomp : 21493760 = 21296876 + 196883 + 1 := by native_decide
theorem factor_196883 : 196883 = 47 * 59 * 71 := by native_decide
theorem factor_21296876 : 21296876 = 2 * 2 * 31 * 41 * 59 * 71 := by native_decide
theorem factor_744 : 744 = 2 * 2 * 2 * 3 * 31 := by native_decide
theorem factor_24 : 24 = 2 * 2 * 2 * 3 := by native_decide
theorem factor_26 : 26 = 2 * 13 := by native_decide

theorem dvd_47_196883 : 47 ∣ 196883 := by native_decide
theorem dvd_59_196883 : 59 ∣ 196883 := by native_decide
theorem dvd_71_196883 : 71 ∣ 196883 := by native_decide
theorem dvd_2_196884 : 2 ∣ 196884 := by native_decide
theorem dvd_3_196884 : 3 ∣ 196884 := by native_decide
theorem dvd_2_24 : 2 ∣ 24 := by native_decide
theorem dvd_3_24 : 3 ∣ 24 := by native_decide
theorem dvd_2_26 : 2 ∣ 26 := by native_decide
theorem dvd_13_26 : 13 ∣ 26 := by native_decide
theorem dvd_2_744 : 2 ∣ 744 := by native_decide
theorem dvd_3_744 : 3 ∣ 744 := by native_decide
theorem dvd_31_744 : 31 ∣ 744 := by native_decide
theorem dvd_2_21296876 : 2 ∣ 21296876 := by native_decide
theorem dvd_31_21296876 : 31 ∣ 21296876 := by native_decide
theorem dvd_41_21296876 : 41 ∣ 21296876 := by native_decide
theorem dvd_59_21296876 : 59 ∣ 21296876 := by native_decide
theorem dvd_71_21296876 : 71 ∣ 21296876 := by native_decide
theorem dvd_2_194 : 2 ∣ 194 := by native_decide

-- ═══ SSP-purity witnesses ═══

theorem ssp_pure_196883 : IsSSPPure 196883 := by
  right; intro p hp hd
  simp [sspPrimes]
  omega

theorem ssp_pure_24 : IsSSPPure 24 := by
  right; intro p hp hd; simp [sspPrimes]; omega

theorem ssp_pure_26 : IsSSPPure 26 := by
  right; intro p hp hd; simp [sspPrimes]; omega

theorem ssp_pure_744 : IsSSPPure 744 := by
  right; intro p hp hd; simp [sspPrimes]; omega

theorem ssp_pure_15 : IsSSPPure 15 := by
  right; intro p hp hd; simp [sspPrimes]; omega

-- ═══ 2-cells ═══

def griessSquare : Cell2 :=
  Cell2.add 196884 196883 1 (by native_decide)

def c2Square : Cell2 :=
  Cell2.add 21493760 21296876 196884 (by native_decide)

-- ═══ Build the complex ═══

def baseZeroCells : List Cell0 :=
  (sspPrimes.map fun p => Cell0.prime p (by simp [sspPrimes]; omega)) ++ [Cell0.unit]

def initComplex : CubicalComplex where
  zeroCells := baseZeroCells
  oneCells := []  -- populated by mkDivPath calls
  twoCells := [griessSquare, c2Square]

end Borcherds.Cubical
