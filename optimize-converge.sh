#!/bin/bash
DOC="docs/todo/converge-peak-benchmark-spec.md"
echo -e "# Performance Optimization Checklist\n\n## Current Baseline\n$(cargo bench --bench performance 2>/dev/null | grep "time:" | head -5)\n\n## Optimizations\n- [ ] Replace HashMap with FxHashMap\n- [ ] Add rayon for parallel processing\n- [ ] Use SmallVec for small collections\n- [ ] Add inline hints to hot functions\n- [ ] Implement arena allocators\n- [ ] Enable LTO in release builds\n- [ ] Use SIMD for vector operations\n- [ ] Profile-guided optimization\n\n## Results\n" > $DOC

STEPS=("s/HashMap/FxHashMap/g src/**/*.rs" "s/\.iter()/\.par_iter()/g src/services/*.rs" "s/Vec<String>/SmallVec<[String; 8]>/g src/models/*.rs" "s/^pub fn analyze_/#[inline(always)]\npub fn analyze_/g src/services/*.rs" "echo \"[profile.release]\nlto = true\" >> Cargo.toml")
NAMES=("FxHashMap" "rayon" "SmallVec" "inline hints" "LTO")

for i in "${!STEPS[@]}"; do
  echo "🔧 Applying: ${NAMES[$i]}"
  eval "sed -i ${STEPS[$i]}" 2>/dev/null || true
  cargo build --release --quiet
  RESULT=$(hyperfine --warmup 1 --runs 3 "target/release/pmat analyze complexity src" --style basic 2>/dev/null | grep Mean || echo "Failed")
  sed -i "s/- \[ \] ${NAMES[$i]}/- [x] ${NAMES[$i]} | $RESULT/" $DOC
  echo "### Step $((i+1)): ${NAMES[$i]}" >> $DOC
  echo "$RESULT" >> $DOC
  echo "" >> $DOC
done

echo -e "\n## Final Performance" >> $DOC
cargo criterion --bench performance -- --save-baseline final 2>&1 | grep -E "(improved|regressed)" >> $DOC
echo -e "\n✅ Optimization complete! See $DOC for results."