// Grouping functions for extract_candidates module
// Included by extract_candidates.rs — no `use` imports allowed here.

/// Extract the name prefix before the first `_` (must be > 2 chars).
fn extract_prefix(name: &str) -> Option<&str> {
    let prefix = name.split('_').next()?;
    if prefix.len() > 2 {
        Some(prefix)
    } else {
        None
    }
}

/// Group functions by name prefix (requires 3+ members per group, functions only).
pub(crate) fn group_by_prefix(results: &[QueryResult]) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in results.iter().enumerate() {
        if r.definition_type != "function" {
            continue;
        }
        if let Some(prefix) = extract_prefix(&r.function_name) {
            groups.entry(prefix.to_string()).or_default().push(i);
        }
    }
    // Only keep groups with 3+ members
    groups.retain(|_, indices| indices.len() >= 3);
    groups
}

/// Group co-located functions that call each other (3+ members).
///
/// Two functions are in the same cluster if they are in the same file
/// and one calls the other (or they share callees).
pub(crate) fn group_by_call_cluster(results: &[QueryResult]) -> HashMap<String, Vec<usize>> {
    let mut by_file: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, r) in results.iter().enumerate() {
        if r.definition_type == "function" {
            by_file.entry(&r.file_path).or_default().push(i);
        }
    }

    let mut clusters: HashMap<String, Vec<usize>> = HashMap::new();
    for (file, indices) in &by_file {
        if indices.len() < 3 {
            continue;
        }
        let names: HashMap<&str, usize> = indices
            .iter()
            .map(|&i| (results[i].function_name.as_str(), i))
            .collect();

        let mut visited = vec![false; indices.len()];
        for (local_idx, &global_idx) in indices.iter().enumerate() {
            if visited[local_idx] {
                continue;
            }
            visited[local_idx] = true;
            let mut cluster = vec![global_idx];
            collect_neighbors(
                &results[global_idx].calls,
                global_idx,
                &names,
                indices,
                &mut visited,
                &mut cluster,
            );
            collect_neighbors(
                &results[global_idx].called_by,
                global_idx,
                &names,
                indices,
                &mut visited,
                &mut cluster,
            );
            if cluster.len() >= 3 {
                let key = format!("{}::cluster_{}", file, results[global_idx].function_name);
                clusters.insert(key, cluster);
            }
        }
    }
    clusters
}

/// Collect unvisited neighbors from a call/caller list into the cluster.
fn collect_neighbors(
    edges: &[String],
    origin: usize,
    names: &HashMap<&str, usize>,
    indices: &[usize],
    visited: &mut [bool],
    cluster: &mut Vec<usize>,
) {
    for edge in edges {
        let Some(&global) = names.get(edge.as_str()) else {
            continue;
        };
        if global == origin {
            continue;
        }
        let local = indices.iter().position(|&i| i == global).unwrap_or(0);
        if !visited[local] {
            visited[local] = true;
            cluster.push(global);
        }
    }
}

/// Find the longest common prefix among a set of strings, trimmed to underscore boundary.
#[allow(dead_code)]
pub(crate) fn longest_common_prefix(names: &[&str]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let first = names[0];
    let len = names[1..]
        .iter()
        .fold(first.len(), |acc, name| common_prefix_len(first, name, acc));
    let prefix = &first[..len];
    match prefix.rfind('_') {
        Some(pos) => first[..pos].to_string(),
        None => prefix.to_string(),
    }
}

fn common_prefix_len(a: &str, b: &str, max: usize) -> usize {
    let limit = max.min(b.len());
    a.bytes()
        .zip(b.bytes())
        .take(limit)
        .position(|(x, y)| x != y)
        .unwrap_or(limit)
}
