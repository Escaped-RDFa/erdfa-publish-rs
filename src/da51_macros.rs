//! DA51 Rust macros — Generate hash functions, shard types, and DASL addresses
//! from Griess cell coordinates (N, M, C) at compile time.

/// The 15 supersingular primes.
pub const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

/// A Griess cell: one of 196,883 hash functions on the Monster lattice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GriessCell {
    pub n: u8, // 1..=71  size
    pub m: u8, // 1..=59  depth
    pub c: u8, // 1..=47  color
}

/// 8D Monster Hash coordinate (visible dimensions).
pub type HashCoord = [u64; 8];

/// 11D full coordinate: 8 visible + 3 compactified (N, M, C).
pub type FullCoord = [u64; 11];

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
        ((self.n as u32 * self.m as u32 * self.c as u32) % 8) as u8
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

    /// Unfold: return the 11D coordinate (8 visible + 3 Calabi-Yau).
    pub fn unfold(&self, visible: &HashCoord) -> FullCoord {
        [
            visible[0], visible[1], visible[2], visible[3],
            visible[4], visible[5], visible[6], visible[7],
            self.n as u64, self.m as u64, self.c as u64,
        ]
    }

    /// Fold: given content hash, produce the 8D visible coords with
    /// (N, M, C) compactified into the first 3 axes.
    pub fn fold(&self, content_hash: &HashCoord) -> HashCoord {
        [
            (content_hash[0] + self.n as u64) % 71,
            (content_hash[1] + self.m as u64) % 59,
            (content_hash[2] + self.c as u64) % 47,
            content_hash[3],
            content_hash[4],
            content_hash[5],
            content_hash[6],
            content_hash[7],
        ]
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

/// Hash function trait — users get a function pointer, internals are private.
pub trait DA51Hash: Send + Sync {
    fn hash(&self, input: &[u8]) -> HashCoord;
    fn cell(&self) -> GriessCell;
    fn name(&self) -> &str;
}

/// Registry: DA51 address → hash function pointer.
/// Plugins register their hash implementations here.
pub type HashRegistry = std::collections::HashMap<DaslAddr, Box<dyn DA51Hash>>;

/// C ABI for hash plugins (.so). Private VMs implement this.
/// erdfa-publish loads via libloading, wraps in Box<dyn DA51Hash>.
#[repr(C)]
pub struct DA51HashFFI {
    pub name: *const u8,
    pub name_len: usize,
    pub n: u8,
    pub m: u8,
    pub c: u8,
    pub hash_fn: extern "C" fn(*const u8, usize, *mut u64),
}

/// Wrapper: turns a C FFI hash into a Rust trait object.
pub struct PluginHash {
    pub ffi: DA51HashFFI,
}

unsafe impl Send for PluginHash {}
unsafe impl Sync for PluginHash {}

impl DA51Hash for PluginHash {
    fn hash(&self, input: &[u8]) -> HashCoord {
        let mut out = [0u64; 8];
        (self.ffi.hash_fn)(input.as_ptr(), input.len(), out.as_mut_ptr());
        out
    }
    fn cell(&self) -> GriessCell {
        GriessCell::new(self.ffi.n, self.ffi.m, self.ffi.c)
    }
    fn name(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ffi.name, self.ffi.name_len)) }
    }
}

/// Generate a DA51 hash closure from Griess cell coordinates.
/// The macro produces a pure function &[u8] → HashCoord.
/// Internal mixing is opaque — swappable via trait impl.
#[macro_export]
macro_rules! da51_hash {
    ($n:expr, $m:expr, $c:expr) => {{
        let cell = GriessCell::new($n, $m, $c);
        move |input: &[u8]| -> HashCoord {
            // Default: SHA-256 mixing, mod SSP reduction, Calabi-Yau fold
            use sha2::{Sha256, Digest};
            let h = Sha256::digest(input);
            let visible = [
                u64::from_le_bytes(h[0..8].try_into().unwrap()) % 71,
                u64::from_le_bytes(h[8..16].try_into().unwrap()) % 59,
                u64::from_le_bytes(h[16..24].try_into().unwrap()) % 47,
                u64::from_le_bytes(h[0..8].try_into().unwrap()) % 31,
                u64::from_le_bytes(h[8..16].try_into().unwrap()) % 23,
                u64::from_le_bytes(h[16..24].try_into().unwrap()) % 13,
                u64::from_le_bytes(h[24..32].try_into().unwrap()) % 11,
                u64::from_le_bytes(h[24..32].try_into().unwrap()) % 7,
            ];
            // Fold (N, M, C) into first 3 axes as Calabi-Yau compactification
            [
                (visible[0] + cell.n as u64) % 71,
                (visible[1] + cell.m as u64) % 59,
                (visible[2] + cell.c as u64) % 47,
                visible[3],
                visible[4],
                visible[5],
                visible[6],
                visible[7],
            ]
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
    fn knowledge_tree_bit_to_tree() {
        // The tree of knowledge: from bit to tree, each level hashed by its Griess cell.
        // Level 0: Bit        (1,1,1)  — smallest unit
        // Level 1: Byte       (8,1,1)  — 8 bits
        // Level 2: Token      (8,2,1)  — byte sequence, 2 steps
        // Level 3: Bigram     (8,2,2)  — 2 tokens, 2 colors
        // Level 4: Line       (16,3,2) — tokens in sequence
        // Level 5: Paragraph  (16,4,3) — lines grouped, trivector
        // Level 6: Section    (32,4,3) — paragraphs grouped
        // Level 7: Document   (64,4,3) — sections grouped
        // Level 8: Corpus     (71,8,3) — documents grouped
        // Level 9: Lattice    (71,59,3) — corpus on orbifold
        // Level 10: Monster   (71,59,47) — full Griess algebra

        let levels: Vec<(&str, GriessCell)> = vec![
            ("bit",       da51_cell!(1, 1, 1)),
            ("byte",      da51_cell!(8, 1, 1)),
            ("token",     da51_cell!(8, 2, 1)),
            ("bigram",    da51_cell!(8, 2, 2)),
            ("line",      da51_cell!(16, 3, 2)),
            ("paragraph", da51_cell!(16, 4, 3)),
            ("section",   da51_cell!(32, 4, 3)),
            ("document",  da51_cell!(64, 4, 3)),
            ("corpus",    da51_cell!(71, 8, 3)),
            ("lattice",   da51_cell!(71, 59, 3)),
            ("monster",   da51_cell!(71, 59, 47)),
        ];

        let content = b"In the beginning was the bit.";

        // Each level hashes the same content with increasing resolution
        let mut prev_hash: Option<HashCoord> = None;
        for (name, cell) in &levels {
            let hasher = da51_hash!(cell.n, cell.m, cell.c);
            let hash = hasher(content);
            let full = cell.unfold(&hash);
            let folded = cell.fold(&hash);

            // Every hash lands on the lattice
            assert!(hash[0] < 71, "{name} orbifold[0] out of range");
            assert!(hash[1] < 59, "{name} orbifold[1] out of range");
            assert!(hash[2] < 47, "{name} orbifold[2] out of range");

            // 11D unfold has the cell coords in positions 8,9,10
            assert_eq!(full[8], cell.n as u64, "{name} unfold N");
            assert_eq!(full[9], cell.m as u64, "{name} unfold M");
            assert_eq!(full[10], cell.c as u64, "{name} unfold C");

            // Folded coords include the cell offset
            assert_eq!(folded[0], (hash[0] + cell.n as u64) % 71, "{name} fold N");

            // Different cells give different hashes (except possible collisions)
            if let Some(prev) = &prev_hash {
                // At least one axis should differ (overwhelmingly likely)
                let same = hash.iter().zip(prev.iter()).filter(|(a,b)| a == b).count();
                assert!(same < 8, "{name} identical to previous level");
            }
            prev_hash = Some(hash);
        }

        // The tree has 11 levels: bit(1,1,1) → monster(71,59,47)
        assert_eq!(levels.len(), 11);

        // The top level IS the Griess algebra
        let top = &levels[10].1;
        assert_eq!(top.n as u64 * top.m as u64 * top.c as u64, 196883);
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
