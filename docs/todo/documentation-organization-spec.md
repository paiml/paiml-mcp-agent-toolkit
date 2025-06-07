### Documentation Workflow

1. **Bug Tracking**:
    - New bugs go in `docs/bugs/active/`
    - When fixed and validated, move to `docs/bugs/archive/`
    - Include resolution notes and commit references

2. **Specification Management**:
    - New specs go in `docs/todo/active/`
    - When implemented and validated, move to `docs/todo/archive/`
    - Include implementation notes and test references

3. **Validation Requirements**:
    - Bugs: Fixed in code, tests pass, documented resolution
    - Specs: Fully implemented, tests pass, matches requirements
    - rust-docs: Generated from latest code, reflects current architecture

4. **Maintenance**:
   ```bash
   # Regenerate Rust documentation
   make docs
   
   # Validate documentation structure
   find . -maxdepth 1 -name "*.md" | grep -v -E "(README|RELEASE_NOTES|CLAUDE)\.md"
   # Should return no results