% eRDFa Campaign Planner — Optimal Cross-Platform Shard Distribution
% Uses real plugin capacities from erdfa-publish registry.
% Parameterized via .dzn data files.

include "globals.mzn";

%% ─── Parameters (from .dzn) ───

int: payload_bytes;                          % raw payload size
int: num_platforms;                          % how many platform types available
int: redundancy_factor;                      % 1 = no redundancy, 2 = 2× copies, etc.
int: max_shards_total;                       % hard cap on total shard count

array[1..num_platforms] of string: platform_name;
array[1..num_platforms] of int: capacity;    % bytes per carrier after 8-byte header
array[1..num_platforms] of int: max_carriers;% max carriers user will use per platform
array[1..num_platforms] of int: cost_microusd;% cost per carrier in micro-USD (1e-6 $)

% ECC: 0=none, 1=hamming(2×), 2=golay(2×)
int: ecc_mode;
int: ecc_expansion = if ecc_mode > 0 then 2 else 1 endif;

%% ─── Derived ───

int: protected_bytes = payload_bytes * ecc_expansion * redundancy_factor;

%% ─── Decision Variables ───

% How many carriers to use on each platform
array[1..num_platforms] of var 0..100: carriers;

%% ─── Constraints ───

% Total capacity must cover the protected payload
constraint sum(p in 1..num_platforms)(carriers[p] * capacity[p]) >= protected_bytes;

% Respect per-platform carrier limits
constraint forall(p in 1..num_platforms)(carriers[p] <= max_carriers[p]);

% Total shards within budget
constraint sum(p in 1..num_platforms)(carriers[p]) <= max_shards_total;

% At least 2 platforms used (distribution = resilience)
constraint count(p in 1..num_platforms)(carriers[p] > 0) >= min(2, num_platforms);

% No single platform carries more than 60% of total capacity
constraint forall(p in 1..num_platforms)(
  carriers[p] * capacity[p] * 100 <= 60 * sum(q in 1..num_platforms)(carriers[q] * capacity[q])
);

%% ─── Objective: minimize total cost ───

var int: total_cost = sum(p in 1..num_platforms)(carriers[p] * cost_microusd[p]);
var int: total_capacity = sum(p in 1..num_platforms)(carriers[p] * capacity[p]);
var int: total_carriers = sum(p in 1..num_platforms)(carriers[p]);

solve minimize total_cost;

%% ─── Output ───

output [
  "═══ eRDFa Campaign Plan ═══\n",
  "Payload:      ", show(payload_bytes), " bytes\n",
  "ECC:          ", if ecc_mode = 0 then "None"
                    elseif ecc_mode = 1 then "Hamming [7,4,3] (M₁₁)"
                    else "Golay [24,12,8] (M₂₄)" endif, "\n",
  "Redundancy:   ", show(redundancy_factor), "×\n",
  "Protected:    ", show(protected_bytes), " bytes\n\n",
  "── Allocation ──\n"
] ++ [
  if fix(carriers[p]) > 0 then
    "  " ++ platform_name[p] ++ ": " ++ show(carriers[p]) ++ " carriers × " ++
    show(capacity[p]) ++ " B = " ++ show(fix(carriers[p]) * capacity[p]) ++ " B" ++
    " ($" ++ show(fix(carriers[p]) * cost_microusd[p]) ++ "µ)\n"
  else "" endif
  | p in 1..num_platforms
] ++ [
  "\n── Summary ──\n",
  "Total carriers: ", show(total_carriers), "\n",
  "Total capacity: ", show(total_capacity), " bytes\n",
  "Overhead:       ", show(total_capacity - protected_bytes), " bytes spare\n",
  "Total cost:     $", show(total_cost), "µ (~$",
    show(total_cost div 1000000), ".", show((total_cost mod 1000000) div 10000), ")\n",
  "Platforms used: ", show(count(p in 1..num_platforms)(carriers[p] > 0)), "\n"
];
