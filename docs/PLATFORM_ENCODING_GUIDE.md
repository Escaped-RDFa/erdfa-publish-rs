# eRDFa Platform Encoding Guide — Stego Pad Style Guide & Tricks

## Architecture

Data → ECC (Hamming/Golay) → Platform Encoder → Carrier (post/tx/video/image)
Carrier → Platform Decoder → ECC Decode → Data

Distribution planner splits payload across N carriers per platform.
MiniZinc optimizer (erdfa-clean/minizinc/optimal_sharding.mzn) finds optimal split.
IPFS manifest tracks all shards with CID + ACL tier (Public/Holder/Private).

---

## Platform Specifications & Encoding Tricks

### Twitter/X (@tweet280)
- **Text limit**: 280 chars (free), 25,000 chars (Premium+)
- **Image**: 1600×900 recommended, max 5MB. JPEG recompressed ~85%.
- **Video**: 1280×720 or 1920×1080, H.264, max 512MB/140s
- **What survives**: Zero-width chars (U+200B ZWSP, U+200C ZWNJ, U+200D ZWJ, U+FEFF BOM) in text. Variation selectors. Alt text on images (1000 chars).
- **What's destroyed**: EXIF metadata stripped. Image recompressed. Video re-encoded.
- **Encoding**: ZWC between visible emoji/text. 2 bits per ZWC char → ~63 bytes/tweet (free), ~5.6KB (Premium+).
- **Trick**: Thread of N tweets = N × 63 bytes. Alt text on images = extra 250 bytes.

### Discord (@discord)
- **Message limit**: 2000 chars (free), 4000 chars (Nitro)
- **Code blocks**: Verbatim preservation inside triple-backtick fences. No recompression.
- **File upload**: 25MB (free), 500MB (Nitro). Files preserved exactly.
- **What survives**: Everything in text. Code blocks are byte-exact. Uploaded files untouched.
- **Encoding**: Hex in code fence = ~980 bytes/message. Or upload .cbor shard directly.
- **Trick**: Bot can post to channel automatically. Webhook = programmatic posting.

### Instagram (@instagram)
- **Caption limit**: 2200 chars
- **Image**: 1080×1080 (square), 1080×1350 (portrait 4:5), 1080×566 (landscape). JPEG recompressed ~72%.
- **Stories/Reels**: 1080×1920 (9:16). Video re-encoded H.264.
- **What survives**: Caption text (ZWC preserved). Image dimensions. Color profile.
- **What's destroyed**: EXIF stripped. JPEG requantized. Video re-encoded.
- **Encoding**: ZWC in caption between hashtags = ~545 bytes. For images: use specific pixel patterns in areas that survive JPEG (low-frequency DCT coefficients).
- **Trick**: Carousel = 10 images × caption = 10 carriers per post.

### TikTok (@tiktok)
- **Description limit**: 2200 chars (was 300, expanded 2024)
- **Video**: 1080×1920 (9:16). Re-encoded H.264/H.265 aggressively.
- **What survives**: Description text (ZWC preserved). Hashtags. Sound name.
- **What's destroyed**: Video completely re-encoded. Metadata stripped.
- **Encoding**: ZWC in description = ~545 bytes/video.
- **Trick**: Series of videos = N × 545 bytes. Comment section also preserves ZWC.

### Solana (@solana-memo + @solana-lamport)
- **Memo instruction**: Up to ~566 bytes UTF-8 per memo instruction. On-chain, immutable.
- **Transaction data**: 1232 bytes max per transaction (entire tx including signatures).
- **Lamport encoding**: Transfer amounts encode data in the last N digits. 10,000 lamports = 10,000 possible values per transfer = ~13 bits per transfer.
- **Mixer wallet trick**: Create N wallets, transfer specific lamport amounts between them. The transfer amounts ARE the data. Each transfer = ~13 bits. 10 transfers = ~130 bits = ~16 bytes.
- **What survives**: Everything. Blockchain is immutable. Memo is permanent. Transfer amounts are permanent.
- **What's destroyed**: Nothing. Ever.
- **Encoding strategies**:
  1. **Memo**: base58 encoded payload, `erdfa:` prefix. 566 bytes/tx.
  2. **Lamport LSB**: last 4 digits of transfer amount = 10,000 values = 13.3 bits/tx. Cost: ~0.00001 SOL/tx.
  3. **Multi-transfer**: N transfers in one tx, each with encoded amount. ~13 bits × N per tx.
  4. **Mixer wallets**: Pre-funded wallets that transfer to each other. Pattern of transfers = data. Plausible deniability.
  5. **Account data**: Rent-exempt account with arbitrary data field. 10KB+ per account. Cost: ~0.07 SOL for 10KB.
- **Cost model**: 10 txns × 5000 lamports base fee = 0.00005 SOL (~$0.01). Data in lamport amounts is essentially free (funds return to you minus fees).

### NFT Tile (@nft-tile)
- **Format**: 512×512 PNG on IPFS. Immutable.
- **Capacity**: 196,608 bytes per tile (6-layer bitplane RGB).
- **71 tiles**: 13.3 MB total (DA51 threshold: need 51/71 to reconstruct).
- **What survives**: Everything. PNG is lossless. IPFS is content-addressed.
- **Encoding**: BitPlane6 — 6 bit planes across R0 G0 B0 R1 G1 B1.
- **Trick**: Tile looks like abstract art. Visually indistinguishable from generative NFT.

### Mastodon (@mastodon)
- **Text limit**: 500 chars (default, server-configurable up to 100K+)
- **Image**: 1920×1080 max, JPEG/PNG preserved better than Twitter.
- **What survives**: ZWC in text. Alt text. Content warnings (CW) text.
- **Encoding**: ZWC = ~120 bytes/post (500 chars). Some instances allow 5000+ chars.
- **Trick**: CW field = extra text capacity. Federated = multiple copies across instances.

### Bluesky (@bluesky)
- **Text limit**: 300 chars (graphemes, not bytes — emoji = 1 char)
- **Image**: 2000×2000 max, JPEG recompressed.
- **What survives**: ZWC in text. Alt text (unclear limit).
- **Encoding**: ZWC = ~68 bytes/post.
- **Trick**: AT Protocol allows custom lexicons — could define erdfa record type.

### GitHub (@github)
- **Commit message**: First line ~72 chars, body unlimited (~65KB practical).
- **Issue/PR body**: 65,536 chars.
- **Gist**: Unlimited file size.
- **What survives**: Everything. Git is content-addressed (SHA-1/SHA-256).
- **Encoding**: Hex in code fence or raw binary in gist. 45KB/commit body.
- **Trick**: Commit to a repo = permanent, signed, timestamped. GPG-signed commits = authenticated carrier.

### Website/IPFS (@website)
- **Capacity**: Effectively unlimited. 50KB practical cap per page.
- **What survives**: Everything you control.
- **Encoding**: Any strategy. Hidden divs, data attributes, ZWC, pixel data.
- **Trick**: Static site on IPFS = immutable + decentralized. erdfa-publish WASM runs client-side.

---

## Image Specifications (for pixel-level stego)

| Platform | Post Size | Aspect | Format | Recompression |
|----------|-----------|--------|--------|---------------|
| Instagram Square | 1080×1080 | 1:1 | JPEG ~72% | Yes, aggressive |
| Instagram Portrait | 1080×1350 | 4:5 | JPEG ~72% | Yes |
| Instagram Story/Reel | 1080×1920 | 9:16 | H.264 | Yes |
| Twitter/X Post | 1600×900 | 16:9 | JPEG ~85% | Yes |
| Twitter/X Card | 800×418 | ~2:1 | JPEG | Yes |
| TikTok Video | 1080×1920 | 9:16 | H.264/H.265 | Yes, aggressive |
| Discord Upload | Any | Any | Preserved | No (if under limit) |
| NFT (IPFS) | 512×512 | 1:1 | PNG lossless | No |
| Bluesky | 2000×2000 max | Any | JPEG | Yes |
| Mastodon | 1920×1080 max | Any | JPEG/PNG | Mild |

---

## Video Specifications

| Platform | Resolution | Codec | Max Duration | Max Size |
|----------|-----------|-------|-------------|----------|
| Twitter/X | 1920×1080 | H.264 | 140s (free), 60min (Premium) | 512MB |
| Instagram Reel | 1080×1920 | H.264 | 90s | ~250MB |
| TikTok | 1080×1920 | H.264/H.265 | 10min | ~287MB |
| YouTube | 3840×2160 | H.264/VP9/AV1 | 12hr | 256GB |
| Discord | Any | Any | Any | 25MB/500MB |

---

## Zero-Width Character Reference

| Char | Unicode | Name | Preserved On |
|------|---------|------|-------------|
| (ZWSP) | U+200B | Zero Width Space | Twitter, Discord, Instagram, TikTok, Mastodon, Bluesky |
| (ZWNJ) | U+200C | Zero Width Non-Joiner | Twitter, Discord, Instagram, TikTok |
| (ZWJ) | U+200D | Zero Width Joiner | All (used in emoji sequences) |
| (BOM) | U+FEFF | Byte Order Mark | Most platforms |
| (IS) | U+2063 | Invisible Separator | Twitter, Discord |
| (IT) | U+2062 | Invisible Times | Twitter, Discord |
| (VS) | U+FE00-FE0F | Variation Selectors | All (part of emoji spec) |

Encoding: 4 ZWC chars = 2 bits each = 1 byte. Capacity = (char_limit - visible_chars) / 4 bytes.

---

## Solana Lamport Encoding Detail

### Strategy 1: LSB Amount Encoding
Transfer X lamports where last N digits encode data.
- 4 digits: 0000-9999 = 13.3 bits per transfer
- 3 digits: 000-999 = 10 bits per transfer
- 2 digits: 00-99 = 6.6 bits per transfer

### Strategy 2: Mixer Wallet Network
1. Create N wallets (N = shard count)
2. Fund each with base amount (e.g., 100,000 lamports)
3. Transfer between wallets with encoded amounts
4. Transfer graph = data (which wallet sends to which, how much)
5. Observer sees normal-looking transfers
6. Decoder knows the wallet set and reads the amounts

### Strategy 3: Account Data
1. Create program-derived address (PDA)
2. Store arbitrary bytes in account data field
3. Rent-exempt: ~0.00089 SOL per byte-year
4. 10KB account = ~0.07 SOL one-time
5. Data is permanent until account is closed

### Cost Estimates (at $150/SOL)
| Method | Capacity | Cost per KB |
|--------|----------|-------------|
| Memo | 566 bytes/tx | ~$0.001 |
| Lamport LSB (4-digit) | ~1.7 bytes/tx | ~$0.006 |
| Account Data | 10KB/account | ~$0.01 |
| Multi-transfer (10/tx) | ~17 bytes/tx | ~$0.001 |

---

## Distribution Plan Examples

### Example: 3 tweets + 10 solana + 3 tiktok + 2 discord
```
target: {tweet: 3, solana: 10, tiktok: 3, discord: 2}
capacity: 3×63 + 10×566 + 3×545 + 2×980 = 9,444 bytes
with golay ecc: ~4,700 bytes usable payload
```

---

## erdfa-publish Plugin Registry (13 plugins)

### Steganographic Carriers
| Plugin | Name | Capacity | Expansion |
|--------|------|----------|-----------|
| PngLsb | png-lsb | unlimited | 9× |
| WavPhase | wav-phase | unlimited | 1× |
| ZeroWidthText | zwc-text | unlimited | 24× |
| RsHexComment | rs-hex | unlimited | 2.1× |
| BitPlane6 | bitplane6 | 196KB/tile | 85× |

### Error-Correcting Codes (EC Zoo)
| Plugin | Name | Correction | Sporadic Group |
|--------|------|-----------|----------------|
| Hamming743 | hamming743 | 1-bit/block | M₁₁ |
| Golay24128 | golay24 | 3-bit/block | M₂₄ (Voyager) |

### Platform Carriers
| Plugin | Name | Capacity | Platform |
|--------|------|----------|----------|
| Tweet280 | tweet280 | 63 bytes | Twitter/X |
| DiscordBlock | discord | 980 bytes | Discord |
| InstaCaption | instagram | 545 bytes | Instagram |
| TikTokDesc | tiktok | 545 bytes | TikTok |
| SolanaMemo | solana-memo | 566 bytes | Solana |
| NftTile | nft-tile | 196KB | IPFS/NFT |

### Composable Chains
golay24 → tweet280 = error-corrected tweet
hamming743 → bitplane6 = error-corrected NFT tile
golay24 → solana-memo = Voyager-grade ECC on-chain

---

## ACL Tiers (IPFS Layers)

| Tier | Who Can Read | How |
|------|-------------|-----|
| Public | Anyone | Plain IPFS gateway |
| Holder | Token holders | Encrypted, key from on-chain balance proof |
| Private | Key holders only | Encrypted, key shared out-of-band |
