#![cfg_attr(coverage_nightly, coverage(off))]

//! Document Index — non-code document search for `pmat query --docs`.
//!
//! Indexes PDF text, SVG labels, image metadata, and markdown/plaintext documents
//! for FTS5-powered search alongside the code function index.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │              Document Index Pipeline                   │
//! ├──────────────────────────────────────────────────────┤
//! │  1. File Walk    │ WalkBuilder finds doc files         │
//! │  2. Extraction   │ PDF/SVG/Image/Markdown → text       │
//! │  3. Chunking     │ Split at pages/headings (4KB max)   │
//! │  4. FTS5 Index   │ documents_fts for BM25 ranking      │
//! │  5. Query        │ --docs flag triggers search          │
//! └──────────────────────────────────────────────────────┘
//! ```

pub(crate) mod build;
pub(crate) mod extractors;
pub(crate) mod sqlite_docs;
pub(crate) mod types;

pub(crate) use build::build_document_index;
pub(crate) use sqlite_docs::{create_documents_schema, query_documents};
pub(crate) use types::DocumentResult;
