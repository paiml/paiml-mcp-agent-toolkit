# pmat closing-keyword lint (PMAT-900001) — ONE snippet, sourced into every
# commit-msg hook pmat writes and by CI's PR-body and commit-body lint.
# GitHub closes an issue when a merged PR body, or a commit that reaches the
# default branch, puts a closing keyword (close/fix/resolve and their -s/-d
# forms, optionally followed by a colon) directly before an issue reference.
# It needs no word boundary a human would read as one: `no-close: #3091`
# closed aprender#3091. The one sanctioned form is a line that starts with
# `Closes #N`; everything after that prefix is still linted.
# The ERE below is src/services/closing_keywords.rs PATTERN, byte for byte, and
# a lib test runs this function and the Rust predicate over one fixture table.
# LC_ALL=C keeps [[:alnum:]] ASCII, as it is in the Rust regex.

# stdin: text. stdout: `N:line` for each line holding a closing reference.
# Exit 0 when at least one line does, 1 when none does.
pmat_closing_keyword_hits() {
    LC_ALL=C sed -E 's/^Closes #[0-9]+//' \
        | LC_ALL=C grep -niE '(^|[^[:alnum:]_])(close[sd]?|fix(e[sd])?|resolve[sd]?):?[[:space:]]*([[:alnum:]_.-]+/[[:alnum:]_.-]+)?#[0-9]+'
}

# $1: a commit message file, as a commit-msg hook receives it. Drops git's
# comment lines and everything below a `git commit -v` scissors line, then
# prints each offending line. Exit 1 when the message would close an issue.
pmat_closing_keyword_commit_msg_lint() {
    local hits
    hits=$(sed '/^# -\{24\} >8 -\{24\}$/,$d' "$1" | grep -v '^#' | pmat_closing_keyword_hits) || return 0
    echo "PMAT commit-msg: this message would close a GitHub issue when it reaches the default branch:" >&2
    printf '%s\n' "$hits" | sed 's/^/  line /' >&2
    echo "  a closing keyword (close/fix/resolve, any tense, optional colon) directly before #N closes #N." >&2
    echo "  to close on purpose, start its own line with:  Closes #N" >&2
    echo "  to mention it, break the adjacency:  fixes issue #N  |  keeps-open #N" >&2
    return 1
}
