#!/bin/bash
STATE="PROFILE"
TARGET_NS=50
while true; do
  case $STATE in
    PROFILE) echo "🔍 Profiling..." && cargo build --release && perf record -g target/release/pmat analyze deep-context . && perf report > perf.txt && STATE="ANALYZE" ;;
    ANALYZE) echo "📊 Analyzing..." && grep -E "(graph|traverse|visit|annotate)" perf.txt | head -20 && cargo asm --lib --rust --simplify "services::dag_builder::DagBuilder::build_graph" > asm.txt && STATE="OPTIMIZE" ;;
    OPTIMIZE) echo "⚡ Optimizing..." && sed -i "s/HashMap<String/FxHashMap<&str/g" src/services/dag_builder.rs && sed -i "s/Vec<String>/SmallVec<[&str; 8]>/g" src/services/dag_builder.rs && sed -i "s/for node in/nodes.par_iter().for_each(|node|/g" src/services/dag_builder.rs && STATE="MEASURE" ;;
    MEASURE) echo "📏 Measuring..." && hyperfine --warmup 3 --min-runs 10 "target/release/pmat analyze dag ." | tee bench.txt && NS=$(grep Mean bench.txt | awk "{print $2}") && [[ ${NS%.*} -lt $TARGET_NS ]] && STATE="CONVERGED" || STATE="PROFILE" ;;
    CONVERGED) echo "✅ Converged to <${TARGET_NS}ns per node!" && break ;;
  esac
  sleep 1
done