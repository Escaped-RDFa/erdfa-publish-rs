"""zkperf — Python decorator for register capture via erdfa_py.

Usage:
    from zkperf import zkperf, zkperf_dump

    @zkperf
    def classify(data: list[int]) -> int:
        return model.predict(data)

    result = classify(input_data)
    zkperf_dump("witnesses.jsonl")
"""
import json, time, functools, inspect, os
import erdfa_py

_witnesses = []
_predictor = None  # FRACTRAN predictor loaded at runtime
_predictions = {"correct": 0, "total": 0, "early_exits": 0}
_cache = {}  # orbifold → result cache (LRU-like, shareable)
_cache_hits = 0
_CACHE_PATH = None


def zkperf_load_predictor(path):
    """Load a trained FRACTRAN predictor for runtime inference."""
    global _predictor
    data = json.load(open(path))
    _predictor = data.get("fractions", [])
    print(f"zkperf: loaded predictor with {len(_predictor)} fractions from {path}")


def zkperf_load_cache(path):
    """Load shared orbifold→result cache from disk."""
    global _cache, _CACHE_PATH
    _CACHE_PATH = path
    try:
        _cache = json.load(open(path))
        print(f"zkperf: loaded {len(_cache)} cached orbifold results from {path}")
    except (FileNotFoundError, json.JSONDecodeError):
        _cache = {}


def zkperf_save_cache(path=None):
    """Save cache to disk for sharing across runs/users."""
    p = path or _CACHE_PATH or "zkperf_cache.json"
    json.dump(_cache, open(p, "w"))
    print(f"zkperf: saved {len(_cache)} cached results → {p}")


def _cache_key(orb):
    return f"{orb[0]},{orb[1]},{orb[2]}"


def _fractran_predict(ssp):
    """Run FRACTRAN predictor on SSP vector. Returns (fires, result_ssp)."""
    if not _predictor:
        return False, ssp
    v = list(ssp)
    for frac in _predictor:
        num, den = frac["num"], frac["den"]
        can_fire = all(v[i] >= den[i] for i in range(min(len(v), len(den))))
        if can_fire:
            for i in range(min(len(v), len(den))):
                v[i] = v[i] - den[i] + num[i]
            return True, v
    return False, v

def zkperf(fn):
    """Decorator: capture registers + type info + DASL address + register lenses."""
    sig = inspect.signature(fn)
    hints = fn.__annotations__ if hasattr(fn, '__annotations__') else {}
    params = list(sig.parameters.keys())
    type_str = ", ".join(f"{p}: {hints.get(p, '?')}" for p in params)
    ret = hints.get('return', '?')
    sig_str = f"({type_str}) -> {ret}"

    # x86_64 SysV ABI: args go in rdi, rsi, rdx, rcx, r8, r9
    ABI_REGS = ["di", "si", "dx", "cx", "r8", "r9"]

    # Build lens: which register holds which typed parameter
    lenses = {}
    for i, p in enumerate(params[:6]):
        reg = ABI_REGS[i]
        ty = hints.get(p, None)
        if ty is None:
            kind = "unknown"
        elif ty in (int,):
            kind = "int"
        elif ty in (float,):
            kind = "float_as_int"  # floats go in xmm, but int-cast visible
        elif ty in (bool,):
            kind = "bitmask"
        elif ty in (str, bytes, bytearray, list, dict, tuple, set):
            kind = "pointer"
        elif hasattr(ty, '__origin__'):  # generic like list[int]
            kind = "pointer"
        else:
            kind = "pointer"  # objects are pointers
        lenses[reg] = {"param": p, "type": str(ty), "kind": kind}
    # ax = return value in post snapshot
    lenses["ax"] = {"param": "return", "type": str(ret), "kind":
        "int" if ret in (int, bool) else "pointer" if ret not in (float, type(None)) else "none"}

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        pre = erdfa_py.capture_regs()
        pre_ssp = erdfa_py.regs_to_ssp(pre)
        pre_orb = erdfa_py.ssp_to_orbifold(pre_ssp)

        # Call stack context
        frame = inspect.currentframe().f_back
        stack = []
        f = frame
        for _ in range(8):
            if f is None: break
            stack.append(f"{f.f_code.co_filename.rsplit('/',1)[-1]}:{f.f_lineno}:{f.f_code.co_name}")
            f = f.f_back
        stack_hash = hash(tuple(stack)) & 0xFFFFFFFF
        fn_hash = hash(fn.__qualname__) & 0xFFFFFFFF

        # Enrich SSP with call context so same function at different call sites differs
        pre_ssp_ctx = list(pre_ssp)
        pre_ssp_ctx[0] = fn_hash % 8           # p2 ← function identity
        pre_ssp_ctx[1] = (fn_hash >> 3) % 8    # p3 ← function identity high
        pre_ssp_ctx[2] = stack_hash % 8         # p5 ← call stack context
        pre_ssp_ctx[3] = (stack_hash >> 3) % 8  # p7 ← call stack context high
        pre_orb_ctx = erdfa_py.ssp_to_orbifold(bytes(pre_ssp_ctx))

        # zkperf context: the full execution context for this call
        ctx = {
            "fn_hash": fn_hash,
            "stack_hash": stack_hash,
            "stack": stack,
            "depth": len(stack),
        }

        # Check cache/predictor with enriched orbifold
        global _cache_hits
        ck = fn.__qualname__ + ":" + _cache_key(pre_orb_ctx)
        early = None
        if ck in _cache:
            _cache_hits += 1
            early = {"ssp": _cache[ck]["ssp"], "orb": tuple(_cache[ck]["orb"]),
                     "fired": True, "source": "cache"}
            _predictions["total"] += 1
        elif _predictor:
            predicted, pred_ssp = _fractran_predict(pre_ssp_ctx)
            if predicted:
                pred_orb = erdfa_py.ssp_to_orbifold(bytes(pred_ssp))
                early = {"ssp": pred_ssp, "orb": pred_orb, "fired": True, "source": "predictor"}
                _predictions["total"] += 1

        t0 = time.monotonic_ns()
        result = fn(*args, **kwargs)
        t1 = time.monotonic_ns()
        post = erdfa_py.capture_regs()
        post_ssp = erdfa_py.regs_to_ssp(post)
        post_orb = erdfa_py.ssp_to_orbifold(post_ssp)

        # Check prediction accuracy
        if early:
            if early["orb"] == post_orb:
                _predictions["correct"] += 1

        # Cache this result for future runs
        _cache[ck] = {"ssp": list(post_ssp), "orb": list(post_orb)}

        tag = f"{fn.__module__}.{fn.__qualname__}|{sig_str}"
        dasl = erdfa_py.dasl_addr(tag.encode())
        w = {
            "fn": fn.__qualname__,
            "module": fn.__module__,
            "sig": sig_str,
            "dasl": dasl,
            "lenses": lenses,
            "ctx": ctx,
            "ts": t0,
            "elapsed_ns": t1 - t0,
            "pre": {"regs": pre, "ssp": list(pre_ssp_ctx), "orb": pre_orb_ctx},
            "post": {"regs": post, "ssp": list(post_ssp), "orb": post_orb},
        }
        if early:
            w["prediction"] = early
        _witnesses.append(w)
        _cache[ck] = {"ssp": list(post_ssp), "orb": list(post_orb)}
        return result
    return wrapper


def zkperf_dump(path="witnesses.jsonl"):
    """Write all captured witnesses to JSONL and save cache."""
    with open(path, "w") as f:
        for w in _witnesses:
            f.write(json.dumps(w) + "\n")
    t = _predictions["total"]
    c = _predictions["correct"]
    e = _predictions["early_exits"]
    acc = f"{c/t:.1%}" if t else "n/a"
    print(f"zkperf: {len(_witnesses)} witnesses → {path}")
    if t:
        print(f"zkperf: predictions {c}/{t} = {acc}, early exits: {e}, cache hits: {_cache_hits}")
    if _cache:
        zkperf_save_cache()


def zkperf_clear():
    _witnesses.clear()
    _predictions["correct"] = 0
    _predictions["total"] = 0
    _predictions["early_exits"] = 0


def zkperf_publish(path="zkperf_shard.cbor", spool=None):
    """Publish witnesses as DA51 CBOR shard via erdfa-publish.

    Each witness becomes a DASL record. The shard is content-addressed
    and can be shared via UUCP/mesh/IPFS.

    Args:
        path: output CBOR file
        spool: erdfa spool dir (default: /tmp/erdfa-spool)
    """
    import struct, hashlib

    if not _witnesses:
        print("zkperf: no witnesses to publish"); return

    spool = spool or os.environ.get("ERDFA_SPOOL", "/tmp/erdfa-spool")
    os.makedirs(spool, exist_ok=True)

    # Build CBOR array of witness records
    # Minimal CBOR: array header + map per witness
    records = []
    for w in _witnesses:
        tag = f"{w.get('module','')}.{w['fn']}|{w.get('sig','')}"
        dasl = w.get("dasl", erdfa_py.dasl_addr(tag.encode()))
        orb_pre = w["pre"]["orb"]
        orb_post = w["post"]["orb"]
        rec = {
            "dasl": dasl,
            "fn": w["fn"],
            "sig": w.get("sig", ""),
            "ts": w["ts"],
            "elapsed_ns": w["elapsed_ns"],
            "ssp_pre": w["pre"]["ssp"],
            "ssp_post": w["post"]["ssp"],
            "orb_pre": list(orb_pre) if isinstance(orb_pre, tuple) else orb_pre,
            "orb_post": list(orb_post) if isinstance(orb_post, tuple) else orb_post,
        }
        if "lenses" in w:
            rec["lenses"] = w["lenses"]
        if "prediction" in w:
            rec["prediction"] = w["prediction"]
        records.append(rec)

    # Wrap as DA51 shard
    shard_data = json.dumps(records).encode()
    shard_hash = erdfa_py.content_hash(shard_data)
    shard_dasl = erdfa_py.dasl_addr(shard_data[:256])
    shard_orb = erdfa_py.orbifold_coords(shard_data[:256])

    shard = {
        "type": "zkperf_witness_shard",
        "version": "0.1.0",
        "dasl": shard_dasl,
        "hash": shard_hash,
        "orbifold": list(shard_orb),
        "count": len(records),
        "records": records,
        "cache_size": len(_cache),
        "predictions": dict(_predictions),
    }

    # Write CBOR (using json as fallback since cbor2 may not be available)
    try:
        import cbor2
        with open(path, "wb") as f:
            cbor2.dump(shard, f)
    except ImportError:
        # Fallback: write as JSON with .cbor extension
        with open(path, "w") as f:
            json.dump(shard, f)

    # Also write to erdfa spool for mesh distribution
    spool_path = os.path.join(spool, f"zkperf_{shard_hash[:16]}.json")
    with open(spool_path, "w") as f:
        json.dump(shard, f)

    # Write eRDFa HTML witness
    html_path = path.replace(".cbor", ".html")
    with open(html_path, "w") as f:
        f.write(f"""<!DOCTYPE html>
<html prefix="da51: https://da51.org/ns# zkperf: https://zkperf.org/ns#">
<head><meta charset="utf-8"><title>zkperf witness {shard_hash[:12]}</title></head>
<body vocab="https://da51.org/ns#">
<div typeof="da51:Shard" resource="#{shard_dasl}">
  <span property="da51:hash">{shard_hash}</span>
  <span property="da51:orbifold">{shard_orb}</span>
  <span property="zkperf:witnesses">{len(records)}</span>
  <span property="zkperf:predictions">{_predictions['correct']}/{_predictions['total']}</span>
  <span property="zkperf:cache">{len(_cache)}</span>
</div>
</body></html>""")

    print(f"zkperf: published {len(records)} witnesses")
    print(f"  CBOR:  {path}")
    print(f"  eRDFa: {html_path}")
    print(f"  spool: {spool_path}")
    print(f"  DASL:  {shard_dasl}")
    print(f"  hash:  {shard_hash}")
    print(f"  orb:   {shard_orb}")


def zkperf_eager(fn):
    """Like @zkperf but skips the function if predictor fires.
    
    Use on pure functions where the orbifold predicts the return value.
    The predicted orbifold mod 10 becomes the digit classification.
    """
    base = zkperf(fn)  # get the instrumented version

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        if _predictor:
            pre = erdfa_py.capture_regs()
            pre_ssp = erdfa_py.regs_to_ssp(pre)
            fired, pred_ssp = _fractran_predict(pre_ssp)
            if fired:
                pred_orb = erdfa_py.ssp_to_orbifold(bytes(pred_ssp))
                _predictions["total"] += 1
                _predictions["early_exits"] += 1
                # Return predicted value (orbifold[0] mod 10 for digit classification)
                return pred_orb[0] % 10
        return base(*args, **kwargs)
    return wrapper
