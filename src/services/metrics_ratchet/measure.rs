//! Running the pinned command of every declared metric.
//!
//! [`super::config`] is pure by design and takes measurements as an argument.
//! This is the module that produces them, and it is where every fail-closed
//! decision about measurement lives: a command that does not run, exits in a
//! way the grep family does not use, prints nothing, or prints something that
//! is not a count becomes [`Measurement::Unavailable`] — never a zero.
//!
//! That distinction is the whole reason `command` is a field of
//! [`super::config::MetricBaseline`] rather than a sentence in a commit
//! message. The scope predicate IS the metric: "the unwrap count of this
//! repository" has been quoted as 570, 11,002, 20,326 and 20,378 within one
//! programme of work by people who each meant a different set of files, two of
//! those differ by 9,324, and one moved by 52 inside a single session. A
//! number nobody can recompute has already rotted.

use super::config::{Measurement, Measurements, MetricBaseline};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// `grep` and `git grep` exit 1 to mean "no matches", which is a legitimate
/// count of zero, not an error. Anything else — 2 from a bad pathspec, 127
/// from a missing binary, 128 from git — is a broken measurement.
const ACCEPTABLE_EXIT_CODES: [i32; 2] = [0, 1];

/// Measure every declared metric by running its own `command`.
pub fn measure_all(
    project_path: &Path,
    metrics: &BTreeMap<String, MetricBaseline>,
) -> Measurements {
    metrics
        .iter()
        .map(|(id, m)| (id.clone(), measure(project_path, &m.command)))
        .collect()
}

/// Run `command` from `project_path` and read a count from its output.
///
/// The shell is `bash -o pipefail`. Both halves matter. `sh` is `dash` on
/// Debian and does not reliably support `pipefail`; and without `pipefail` the
/// canonical `<producer> | wc -l` shape reports `wc`'s status, so a producer
/// that failed outright reads as a clean `0` — which a ratchet, looking only
/// upward, greets as the largest improvement in the project's history and the
/// lowering job then makes permanent.
pub fn measure(project_path: &Path, command: &str) -> Measurement {
    if command.trim().is_empty() {
        return Measurement::Unavailable(
            "the metric declares no command, so its baseline cannot be reproduced".into(),
        );
    }
    let output = Command::new("bash")
        .arg("-o")
        .arg("pipefail")
        .arg("-c")
        .arg(command)
        .current_dir(project_path)
        // Deterministic collation and message text: a metric that greps must
        // not depend on the locale of whoever ran it.
        .env("LC_ALL", "C")
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Measurement::Unavailable(format!("could not run bash: {e}")),
    };

    match output.status.code() {
        Some(c) if ACCEPTABLE_EXIT_CODES.contains(&c) => {}
        Some(c) => {
            return Measurement::Unavailable(format!(
                "command exited {c}: {}",
                first_line(&String::from_utf8_lossy(&output.stderr))
            ))
        }
        None => return Measurement::Unavailable("command was killed by a signal".into()),
    }

    parse_count(&String::from_utf8_lossy(&output.stdout))
}

/// The last non-empty line of `stdout`, parsed as a count.
///
/// Last rather than first: the `| wc -l` idiom puts the answer at the end, and
/// a command that also prints progress must not be silently misread.
pub fn parse_count(stdout: &str) -> Measurement {
    let Some(line) = stdout.lines().rev().find(|l| !l.trim().is_empty()) else {
        return Measurement::Unavailable("command printed nothing".into());
    };
    match line.trim().parse::<i64>() {
        Ok(v) => Measurement::Value(v),
        Err(_) => Measurement::Unavailable(format!(
            "command printed `{}`, which is not a count",
            line.trim()
        )),
    }
}

fn first_line(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("(no stderr)")
        .trim()
        .to_string()
}
