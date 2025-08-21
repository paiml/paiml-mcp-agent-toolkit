#!/bin/bash
# Archive outdated documentation per PMAT-3004
# This script moves outdated docs to appropriate archive locations

set -e

DOCS_DIR="docs"
ARCHIVE_DIR="docs/archive"

echo "📚 Archiving outdated documentation..."

# Create archive structure if not exists
mkdir -p "$ARCHIVE_DIR/pre-v2.0"/{api-docs,implementation-docs,legacy-features,old-guides}
mkdir -p "$ARCHIVE_DIR/deprecated"/{scripts,configs,templates}

# Function to archive file with message
archive_file() {
    local src="$1"
    local dest="$2"
    if [ -f "$src" ]; then
        echo "  📦 Archiving: $(basename "$src")"
        mv "$src" "$dest" 2>/dev/null || true
    fi
}

# Archive old API documentation
archive_file "$DOCS_DIR/api-guide.md" "$ARCHIVE_DIR/pre-v2.0/api-docs/"

# Archive old implementation docs
archive_file "$DOCS_DIR/implementation-summary.md" "$ARCHIVE_DIR/pre-v2.0/implementation-docs/"

# Archive old templates that are not actively used
if [ -d "$DOCS_DIR/templates" ]; then
    for template in "$DOCS_DIR/templates"/*.md; do
        if [[ $(basename "$template") != "README.md" ]]; then
            archive_file "$template" "$ARCHIVE_DIR/deprecated/templates/"
        fi
    done
fi

# Archive analysis docs older than v2.0
if [ -d "$DOCS_DIR/analysis" ]; then
    echo "  📦 Archiving old analysis documents..."
    mv "$DOCS_DIR/analysis" "$ARCHIVE_DIR/pre-v2.0/implementation-docs/" 2>/dev/null || true
fi

# Archive old update documents
if [ -d "$DOCS_DIR/updates" ]; then
    echo "  📦 Archiving update documents..."
    mkdir -p "$ARCHIVE_DIR/pre-v2.0/updates"
    for update in "$DOCS_DIR/updates"/*.md; do
        archive_file "$update" "$ARCHIVE_DIR/pre-v2.0/updates/"
    done
    rmdir "$DOCS_DIR/updates" 2>/dev/null || true
fi

# Archive old cargo docs
archive_file "$DOCS_DIR/cargo-examples-guide.md" "$ARCHIVE_DIR/pre-v2.0/old-guides/"
archive_file "$DOCS_DIR/cargo-publishing.md" "$ARCHIVE_DIR/pre-v2.0/old-guides/"

# Archive MCP v1 docs (we're on pmcp 1.0 now)
archive_file "$DOCS_DIR/mcp-methods.md" "$ARCHIVE_DIR/pre-v2.0/api-docs/"
archive_file "$DOCS_DIR/mcp-stateful-server.md" "$ARCHIVE_DIR/pre-v2.0/implementation-docs/"
archive_file "$DOCS_DIR/mcp-claude-code-setup.md" "$ARCHIVE_DIR/pre-v2.0/old-guides/"

# Archive old PDMT docs (integrated into main docs now)
archive_file "$DOCS_DIR/pdmt-detailed-examples.md" "$ARCHIVE_DIR/pre-v2.0/old-guides/"
archive_file "$DOCS_DIR/pdmt-integration-guide.md" "$ARCHIVE_DIR/pre-v2.0/old-guides/"

# Archive old quality docs (superseded by new quality gate system)
archive_file "$DOCS_DIR/quality-fixes-using-pdmt-mcp-proxy.md" "$ARCHIVE_DIR/pre-v2.0/implementation-docs/"
archive_file "$DOCS_DIR/quality-gates-proxy-detailed.md" "$ARCHIVE_DIR/pre-v2.0/implementation-docs/"

# Clean up empty directories
find "$DOCS_DIR" -type d -empty -delete 2>/dev/null || true

echo "✅ Documentation archival complete!"
echo ""
echo "📊 Archive Summary:"
echo "  - Pre-v2.0 docs moved to: $ARCHIVE_DIR/pre-v2.0/"
echo "  - Deprecated items moved to: $ARCHIVE_DIR/deprecated/"
echo "  - Archive index available at: $ARCHIVE_DIR/ARCHIVE_INDEX.md"
echo ""
echo "📝 Next steps:"
echo "  1. Review archived files in $ARCHIVE_DIR"
echo "  2. Update any broken links in active documentation"
echo "  3. Commit changes with PMAT-3004"