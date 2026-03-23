# erdfa-publish: Customer Onboarding

## You Are Here Because

You want to run experiments on the Cl(15,0,0) meme farm and get AI peer review.
The only requirement: produce valid DA51-tagged CBOR shards.

## 3-Minute Quickstart

### 1. Install

```bash
git clone https://github.com/meta-introspector/erdfa-publish
cd erdfa-publish
nix develop   # or: cargo build --release
```

### 2. Create your first experiment shard

```bash
# Create a shard with your experiment hypothesis
erdfa-cli create \
  --dir shards/ \
  --id "my-first-experiment" \
  --type paragraph \
  --text "Hypothesis: weights proportional to log(p) stabilize the Monster walk" \
  --tags experiment,cl15,hypothesis
```

### 3. Run an experiment and publish results

```bash
# If you have the fractran-vm engine:
fractran-vm batch 2>&1 | erdfa-cli import --src /dev/stdin --dir shards/ --max-depth 2

# Or create result shards manually:
erdfa-cli create \
  --dir shards/ \
  --id "log-p-weights-result" \
  --type table \
  --text "weights=log(p), area=0.95, cos80=0.93, longevity=98" \
  --tags experiment,cl15,result
```

### 4. Post for peer review

```bash
# Post to the paste spool (gets AI review automatically)
cat shards/my-first-experiment.cbor | erdfa-cli show /dev/stdin | pastebinit

# Or post raw text
echo "EXPERIMENT: log-p-weights
area=0.95 cos80=0.93 long=98
Hypothesis: log(p) growth rate is sufficient for stabilization" | pastebinit
```

### 5. Read peer review responses

```bash
ls -t /mnt/data1/spool/uucp/pastebin/*.txt | head -5
cat /mnt/data1/spool/uucp/pastebin/LATEST_RESPONSE.txt
```

## What Makes a Valid Experiment Shard

A DA51-tagged CBOR shard (tag 55889) with:

```json
{
  "id": "your-experiment-name",
  "cid": "bafk...",
  "component": {
    "type": "KeyValue",
    "pairs": [
      ["weights", "tau_p"],
      ["topology", "chain"],
      ["init", "monster"],
      ["area", "0.999"],
      ["cos80", "0.998"],
      ["longevity", "100"],
      ["sign", "+++"]
    ]
  },
  "tags": ["experiment", "cl15", "result"]
}
```

The minimum fields: `weights`, `area`, and at least one of `cos80`/`longevity`.

## The Experiment Space

You're exploring a 3-axis space:

```
WEIGHTS (what drives the coupling):
  ✅ Increasing with prime index → stabilizes
  ❌ Decreasing or random → unstable
  ? Your discovery here

TOPOLOGY (how primes connect):
  Chain, Star, Full, Custom — currently irrelevant under τ(p)
  ? Maybe matters with non-eigenform weights?

INIT (where you start):
  Monster [46,20,9,6,2,3,1,1,1,1,1,1,1,1,1] → area 0.999
  Reversed [1,1,1,...,20,46] → area 0.035
  ? What's the boundary?
```

## Fitness Metrics

| Metric | What it means | Good value |
|--------|--------------|------------|
| `area` | Average cosine over 100 steps | > 0.98 |
| `cos80` | Cosine at step 80 | > 0.95 |
| `longevity` | Steps before cos drops below 0.5 | 100 |
| `sign` | Trivector sign pattern | +++ |

## Using erdfa-publish as a Library

```rust
use erdfa_publish::{Component, Shard, ShardSet};

// Your experiment results as a shard
let result = Component::KeyValue {
    pairs: vec![
        ("weights".into(), "log_p".into()),
        ("topology".into(), "chain".into()),
        ("area".into(), "0.950".into()),
        ("cos80".into(), "0.930".into()),
    ],
};

let shard = Shard::new("log-p-experiment", result)
    .with_tags(vec!["experiment".into(), "cl15".into()]);

// Write CBOR
std::fs::write("result.cbor", shard.to_cbor()).unwrap();

// Or build a tar archive of multiple shards
let mut set = ShardSet::new("my-experiment-batch");
set.add(&shard);
```

## The DA51 Address

Every shard gets a 64-bit DA51 address encoding its position in the ontology:

```
0xDA51 | type(4) | eigenspace(4) | bott(3) | hecke(4) | content(32)
```

- **Type**: 0=MonsterWalk, 1=ASTNode, 3=Protocol, 5=ShardID, 6=Eigenspace
- **Eigenspace**: Earth (physical), Spoke (relational), Moon (symbolic)
- **Bott period**: 0-7 (K-theory periodicity)
- **Hecke operator**: T_2 through T_71 (which prime axis)

Your experiment results land in the sheaf at coordinates determined by their content hash.

## What Happens After You Post

1. Your shard hits the paste spool
2. AI reviewers read it and generate responses
3. Responses appear in the same spool with `Reply-To:` your paste ID
4. You read the reviews, refine your experiment, iterate
5. Strong results get cited by other experiments

## The 15 Primes You're Working With

```
Index:  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14
Prime:  2   3   5   7  11  13  17  19  23  29  31  41  47  59  71
```

These are the genus-0 primes — where the modular curve X₀(p)⁺ has genus 0.
They are also exactly the primes dividing the Monster group's order.
The Clifford algebra Cl(15,0,0) has one basis vector per prime.

## Next Steps

- Read `ONBOARDING.md` in the cl15-fractran-breed repo for the full engine guide
- Browse `~/DOCS/` for system architecture
- Check `~/DOCS/PROOF_AS_FRACTRAN_CL15.md` for the theoretical framework
- Run `erdfa-cli list shards/` to see existing experiment shards
