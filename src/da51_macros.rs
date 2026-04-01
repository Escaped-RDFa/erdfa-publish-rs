//! DA51 Rust macros — Generate hash functions, shard types, and DASL addresses
//! from Griess cell coordinates (N, M, C) at compile time.

/// The 15 supersingular primes.
pub const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

/// A Griess cell: one of 196,883 hash functions on the Monster lattice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GriessCell {
    pub n: u8, // 1..=71  size: bytes per FRACTRAN step
    pub m: u8, // 1..=59  depth: number of FRACTRAN steps
    pub c: u8, // 1..=47  color: SSP prime subset touched
}

/// 8D Monster Hash coordinate.
pub type HashCoord = [u64; 8];

/// DASL address (8 bytes).
pub type DaslAddr = u64;

impl GriessCell {
    pub const fn new(n: u8, m: u8, c: u8) -> Self {
        Self { n, m, c }
    }

    pub const fn dasl(&self) -> DaslAddr {
        (0xDA51u64 << 48)
            | ((self.n as u64) << 36)
            | ((self.m as u64) << 28)
            | ((self.c as u64) << 20)
    }

    pub const fn orbifold(&self) -> (u8, u8, u8) {
        (self.n, self.m, self.c)
    }

    pub const fn bott(&self) -> u8 {
        ((self.n as u16 * self.m as u16 * self.c as u16) % 8) as u8
    }

    pub const fn hecke_index(&self) -> usize {
        ((self.n as usize * self.m as usize * self.c as usize) % 15)
    }

    pub const fn grade(&self) -> u8 {
        let product = self.n as u64 * self.m as u64 * self.c as u64;
        let mut g = 0u8;
        let mut i = 0;
        while i < 15 {
            if product % SSP[i] == 0 { g += 1; }
            i += 1;
        }
        g
    }
}

/// Compile-time Griess cell from literal coordinates.
#[macro_export]
macro_rules! da51_cell {
    ($n:expr, $m:expr, $c:expr) => {
        GriessCell::new($n, $m, $c)
    };
}

/// Compile-time DASL address from Griess coordinates.
#[macro_export]
macro_rules! da51_addr {
    ($n:expr, $m:expr, $c:expr) => {
        GriessCell::new($n, $m, $c).dasl()
    };
}

/// Generate a FRACTRAN hash function from Griess cell coordinates.
/// The macro expands to a closure that hashes &[u8] → HashCoord.
#[macro_export]
macro_rules! da51_hash {
    ($n:expr, $m:expr, $c:expr) => {{
        let cell = GriessCell::new($n, $m, $c);
        move |input: &[u8]| -> HashCoord {
            // Encode input as integer via chunk multiplication
            let chunk_size = cell.n as usize;
            let mut state: u128 = 1;
            for chunk in input.chunks(chunk_size.max(1)) {
                let mut v: u128 = 0;
                for (i, &b) in chunk.iter().enumerate() {
                    v |= (b as u128) << (i * 8);
                }
                state = state.wrapping_mul(v.wrapping_add(1));
            }
            // Apply M FRACTRAN steps using C color primes
            let depth = cell.m as usize;
            let colors = cell.c as usize;
            for _step in 0..depth {
                for ci in 0..colors.min(15) {
                    let p = SSP[ci] as u128;
                    if state % p == 0 {
                        // FRACTRAN fraction: multiply by next prime / this prime
                        let next = SSP[(ci + 1) % 15] as u128;
                        state = state / p * next;
                        break;
                    }
                }
            }
            // Reduce to 8D Monster lattice
            let s = state as u64;
            [s % 71, s % 59, s % 47, s % 31, s % 23, s % 13, s % 11, s % 7]
        }
    }};
}

/// Generate a DA51 shard type with embedded Griess cell metadata.
#[macro_export]
macro_rules! da51_shard_type {
    ($name:ident, $n:expr, $m:expr, $c:expr) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            pub content: Vec<u8>,
            pub cell: GriessCell,
            pub hash: HashCoord,
            pub dasl: DaslAddr,
        }

        impl $name {
            pub const CELL: GriessCell = da51_cell!($n, $m, $c);
            pub const DASL: DaslAddr = da51_addr!($n, $m, $c);

            pub fn new(content: Vec<u8>) -> Self {
                let hasher = da51_hash!($n, $m, $c);
                let hash = hasher(&content);
                Self {
                    content,
                    cell: Self::CELL,
                    hash,
                    dasl: Self::DASL,
                }
            }

            pub fn orbifold(&self) -> (u64, u64, u64) {
                (self.hash[0], self.hash[1], self.hash[2])
            }
        }
    };
}

// ── Named hash functions from the Griess lattice ────────────────

/// Type 1: Bootstrap hash (1 byte, 1 step, 1 color)
pub const BOOTSTRAP: GriessCell = da51_cell!(1, 1, 1);

/// Type 3: Borcherds hash (8 bytes, 4 steps, 3 trivector primes)
pub const BORCHERDS: GriessCell = da51_cell!(8, 4, 3);

/// Type 6: Composite (16 bytes, 8 steps, 3 colors) — current default
pub const COMPOSITE: GriessCell = da51_cell!(16, 8, 3);

/// Monster-complete (71 bytes, 59 steps, 47 colors) — maximum resolution
pub const MONSTER_COMPLETE: GriessCell = da51_cell!(71, 59, 47);

// ── Shard types generated from cells ────────────────────────────

da51_shard_type!(BootstrapShard, 1, 1, 1);
da51_shard_type!(BorcherdsShard, 8, 4, 3);
da51_shard_type!(CompositeShard, 16, 8, 3);
da51_shard_type!(MonsterShard, 71, 59, 47);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn griess_cell_basics() {
        let cell = da51_cell!(8, 4, 3);
        assert_eq!(cell.orbifold(), (8, 4, 3));
        assert_eq!(cell.bott(), (8 * 4 * 3) as u8 % 8); // 96 % 8 = 0
        assert!(cell.dasl() >> 48 == 0xDA51);
    }

    #[test]
    fn griess_product() {
        assert_eq!(71u64 * 59 * 47, 196883);
    }

    #[test]
    fn hash_deterministic() {
        let hasher = da51_hash!(8, 4, 3);
        let a = hasher(b"moonshine");
        let b = hasher(b"moonshine");
        assert_eq!(a, b);
    }

    #[test]
    fn different_cells_different_hashes() {
        let h1 = da51_hash!(1, 1, 1);
        let h2 = da51_hash!(8, 4, 3);
        assert_ne!(h1(b"test"), h2(b"test"));
    }

    #[test]
    fn shard_type_works() {
        let s = BorcherdsShard::new(b"196883".to_vec());
        assert_eq!(BorcherdsShard::CELL, da51_cell!(8, 4, 3));
        assert!(BorcherdsShard::DASL >> 48 == 0xDA51);
        let (o0, o1, o2) = s.orbifold();
        assert!(o0 < 71 && o1 < 59 && o2 < 47);
    }
}
