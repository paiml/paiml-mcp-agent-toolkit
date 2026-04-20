# KAIZEN-0014: Re-ranking BM25 query results with graph-adjacency boost

**Source paper:** arxiv:2510.04905 — "Retrieval-Augmented Code Generation: A Survey with Focus on Repository-Level Approaches" (Oct 2025) + Dropstone blog on 40-language tree-sitter scaling (2025)
**Category:** performance
**Priority:** low
**Effort:** S

## Problem
`pmat query` uses FTS5 BM25 (KAIZEN-era SQLite migration) for lexical ranking, plus semantic embeddings for ranking. The RAG survey observes that *repository-level* systems benefit from combining BM25 with a structural-adjacency bonus (how reachable is the result from the seed term's symbol graph?). Our current re-rank does not exploit the graph.

## Proposed improvement
After BM25 + embedding fusion, apply a graph-adjacency re-rank: boost results whose symbol is within `k` CALLS-graph hops of any term matched literally in the query (e.g., if the query is "tokenize" and we BM25-match `Tokenizer::new`, then `Tokenizer::feed` gets a boost).

## Impact
- RAG survey cites 5–15% precision lift on repo-level queries from hybrid-rank approaches.
- Inherits natively from trueno-graph's O(1) adjacency lookup.

## Implementation sketch
1. Identify *anchor symbols* in the query (literal-matched function names).
2. Expand `k=2` hops in CALLS graph, produce `boost_set`.
3. Multiply final score by `1 + 0.15` if result is in `boost_set`, `1 + 0.05` at k=2.
4. Feature-flag behind `--graph-rerank` initially.

## Acceptance criteria
- `pmat query "tokenize" --graph-rerank` shifts results; informal dogfood A/B shows preferred ordering.
- Performance: re-rank adds <20 ms to typical queries (graph adjacency is already O(1)).
- **No MCP schema change** — internal ranking only.
