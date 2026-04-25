use crate::model::{Diagnostic, IssueKind, ResolutionCandidate};

pub(super) fn diagnostics(command: &str, candidates: &[ResolutionCandidate]) -> Vec<Diagnostic> {
    let Some(winner) = candidates.iter().find(|candidate| candidate.wins) else {
        return Vec::new();
    };

    candidates
        .iter()
        .filter(|candidate| !candidate.wins && candidate.entry_index > winner.entry_index)
        .map(|candidate| shadowed_diagnostic(command, winner, candidate))
        .collect()
}

fn shadowed_diagnostic(
    command: &str,
    winner: &ResolutionCandidate,
    candidate: &ResolutionCandidate,
) -> Diagnostic {
    Diagnostic {
        kind: IssueKind::ShadowedCommand,
        message: format!(
            "{} at {} is shadowed by {}",
            command, candidate.path, winner.path
        ),
        entry_index: Some(candidate.entry_index),
        entry_value: Some(candidate.path.clone()),
        related_indexes: vec![winner.entry_index, candidate.entry_index],
    }
}
