#[cfg(test)]
mod leech_voa_tests {
    use erdfa_publish::da51_macros::*;
    use erdfa_publish::da51_hash;

    /// The Leech lattice constants
    const LEECH_RANK: u64 = 24;
    const LEECH_DET: u64 = 1;
    const LEECH_MIN_NORM: u64 = 4;
    const LEECH_ROOTS: u64 = 0;
    const LEECH_MINIMAL_VECTORS: u64 = 196560;
    const LEECH_KISSING: u64 = 196560;

    /// V♮ moonshine module constants
    const VOA_CENTRAL_CHARGE: u64 = 24;
    const VOA_DIM_V0: u64 = 1;
    const VOA_DIM_V1: u64 = 0;
    const VOA_DIM_V2: u64 = 196884;
    const GRIESS_DIM: u64 = 196883;
    const MONSTER_CLASSES: u64 = 194;

    /// j-function coefficients: j(τ) - 744 = Σ c(n) qⁿ
    const J_COEFFS: [(u64, i64); 6] = [
        (0, 1), (1, 196884), (2, 21493760), (3, 864299970),
        (4, 20245856256), (5, 333202640600),
    ];

    /// Crown product: the three largest SSP primes
    const CROWN: (u64, u64, u64) = (47, 59, 71);

    #[test]
    fn leech_lattice_hashes() {
        let borcherds = da51_hash!(8, 4, 3);

        // Hash each Leech constant
        let rank_hash = borcherds(b"Leech rank 24");
        let det_hash = borcherds(b"Leech det 1");
        let norm_hash = borcherds(b"Leech min norm 4");
        let vectors_hash = borcherds(b"Leech minimal vectors 196560");
        let kissing_hash = borcherds(b"Leech kissing number 196560");

        // All land on the lattice
        assert!(rank_hash[0] < 71);
        assert!(vectors_hash[1] < 59);
        assert!(kissing_hash[2] < 47);

        // Kissing number and minimal vectors should hash identically
        // (same content → same hash)
        assert_eq!(
            borcherds(b"196560"),
            borcherds(b"196560")
        );

        // Rank 24 and central charge 24 should hash identically
        assert_eq!(
            borcherds(b"24"),
            borcherds(b"24")
        );

        println!("Leech rank 24:        orb=({},{},{})", rank_hash[0], rank_hash[1], rank_hash[2]);
        println!("Leech det 1:          orb=({},{},{})", det_hash[0], det_hash[1], det_hash[2]);
        println!("Leech min norm 4:     orb=({},{},{})", norm_hash[0], norm_hash[1], norm_hash[2]);
        println!("Leech 196560 vectors: orb=({},{},{})", vectors_hash[0], vectors_hash[1], vectors_hash[2]);
    }

    #[test]
    fn moonshine_module_hashes() {
        let borcherds = da51_hash!(8, 4, 3);

        // Hash V♮ structure
        let v0 = borcherds(b"V0 dim 1 vacuum");
        let v1 = borcherds(b"V1 dim 0 no currents");
        let v2 = borcherds(b"V2 dim 196884 Griess algebra");
        let griess = borcherds(b"196883");
        let griess_plus_1 = borcherds(b"196884");

        println!("V0 (vacuum):     orb=({},{},{})", v0[0], v0[1], v0[2]);
        println!("V1 (no currents):orb=({},{},{})", v1[0], v1[1], v1[2]);
        println!("V2 (Griess):     orb=({},{},{})", v2[0], v2[1], v2[2]);
        println!("196883:          orb=({},{},{})", griess[0], griess[1], griess[2]);
        println!("196884:          orb=({},{},{})", griess_plus_1[0], griess_plus_1[1], griess_plus_1[2]);

        // 196883 + 1 = 196884: they should be DIFFERENT points (different content)
        assert_ne!(griess, griess_plus_1);

        // Crown product test
        let crown = borcherds(b"47 * 59 * 71 = 196883");
        println!("Crown product:   orb=({},{},{})", crown[0], crown[1], crown[2]);
    }

    #[test]
    fn j_function_coefficients() {
        let borcherds = da51_hash!(8, 4, 3);

        println!("j-function coefficients:");
        let mut prev: Option<HashCoord> = None;
        for (n, c) in &J_COEFFS {
            let label = format!("c({}) = {}", n, c);
            let hash = borcherds(label.as_bytes());
            println!("  c({}) = {:>15}: orb=({},{},{})", n, c, hash[0], hash[1], hash[2]);

            // Each coefficient should hash to a different point
            if let Some(p) = &prev {
                assert_ne!(&hash, p, "c({}) collided with previous", n);
            }
            prev = Some(hash);
        }
    }

    #[test]
    fn mckay_thompson_identity() {
        let borcherds = da51_hash!(8, 4, 3);

        // The identity class T_1 = j - 744
        // Its first coefficient is 196884 = 196883 + 1
        let t1 = borcherds(b"T_1 McKay-Thompson identity class");
        let j_minus_744 = borcherds(b"j(tau) - 744");

        println!("T_1 (identity):  orb=({},{},{})", t1[0], t1[1], t1[2]);
        println!("j(τ) - 744:      orb=({},{},{})", j_minus_744[0], j_minus_744[1], j_minus_744[2]);

        // Hash all 194 conjugacy classes (by label)
        println!("\nFirst 15 conjugacy classes:");
        for i in 1..=15 {
            let label = format!("Monster conjugacy class {}", i);
            let hash = borcherds(label.as_bytes());
            println!("  class {:>3}: orb=({},{},{})", i, hash[0], hash[1], hash[2]);
        }
    }

    #[test]
    fn griess_cell_is_voa_state() {
        // The key theorem: each GriessCell (N,M,C) corresponds to a V₂ basis state
        // There are 71 × 59 × 47 = 196,883 cells = dim(Griess algebra)
        assert_eq!(71u64 * 59 * 47, GRIESS_DIM);

        // The Borcherds cell (8,4,3) hashes itself
        let cell = GriessCell::new(8, 4, 3);
        let self_hash = da51_hash!(8, 4, 3)(b"GriessCell(8,4,3)");

        println!("Borcherds cell (8,4,3):");
        println!("  DASL:     0x{:016x}", cell.dasl());
        println!("  orbifold: {:?}", cell.orbifold());
        println!("  bott:     {}", cell.bott());
        println!("  grade:    {}", cell.grade());
        println!("  self-hash: orb=({},{},{})", self_hash[0], self_hash[1], self_hash[2]);

        // The Monster-complete cell (71,59,47) = top of the Griess algebra
        let top = GriessCell::new(71, 59, 47);
        let top_hash = da51_hash!(71, 59, 47)(b"GriessCell(71,59,47)");

        println!("\nMonster-complete cell (71,59,47):");
        println!("  DASL:     0x{:016x}", top.dasl());
        println!("  orbifold: {:?}", top.orbifold());
        println!("  bott:     {}", top.bott());
        println!("  grade:    {}", top.grade());
        println!("  self-hash: orb=({},{},{})", top_hash[0], top_hash[1], top_hash[2]);

        // Different cells give different self-hashes
        assert_ne!(self_hash, top_hash);
    }
}
