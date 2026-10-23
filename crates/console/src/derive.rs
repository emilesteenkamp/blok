//! Pure derivations from a [`RunEvent`] history — kept separate from the
//! [`crate::Console`] component so they're unit-testable without rendering.

use runtime_protocol::RunEvent;

/// Every [`RunEvent::Output`] line seen so far, in order.
pub(crate) fn output_lines(events: &[RunEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            RunEvent::Output(line) => Some(line.as_str()),
            _ => None,
        })
        .collect()
}

/// Whether the program is currently suspended on a
/// [`RunEvent::InputRequested`] with no answer yet — true exactly when
/// that's the most recent event, since any later event (more output, a
/// pause, finishing, erroring) implies it was already answered.
pub(crate) fn is_waiting_for_input(events: &[RunEvent]) -> bool {
    matches!(events.last(), Some(RunEvent::InputRequested))
}

/// Whether the program is still actively executing — more output could
/// still arrive without any action from the user. False once it's
/// finished, errored, or is waiting on input (that state gets its own
/// input-row affordance, so it doesn't need the output pane's cursor too).
pub(crate) fn is_still_running(events: &[RunEvent]) -> bool {
    matches!(
        events.last(),
        Some(RunEvent::Output(_) | RunEvent::Paused { .. })
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_lines_collects_only_output_events_in_order() {
        let events = vec![
            RunEvent::Output("first".to_string()),
            RunEvent::Paused {
                block: block_ir::BlockId::from_raw(0),
                locals: vec![],
            },
            RunEvent::Output("second".to_string()),
        ];
        assert_eq!(output_lines(&events), vec!["first", "second"]);
    }

    #[test]
    fn no_events_means_not_waiting() {
        assert!(!is_waiting_for_input(&[]));
    }

    #[test]
    fn trailing_input_request_means_waiting() {
        let events = vec![RunEvent::Output("hi".to_string()), RunEvent::InputRequested];
        assert!(is_waiting_for_input(&events));
    }

    #[test]
    fn output_after_an_input_request_means_answered() {
        let events = vec![
            RunEvent::InputRequested,
            RunEvent::Output("thanks".to_string()),
        ];
        assert!(!is_waiting_for_input(&events));
    }

    #[test]
    fn no_events_means_not_running() {
        assert!(!is_still_running(&[]));
    }

    #[test]
    fn trailing_output_means_still_running() {
        let events = vec![RunEvent::Output("working...".to_string())];
        assert!(is_still_running(&events));
    }

    #[test]
    fn trailing_paused_means_still_running() {
        let events = vec![RunEvent::Paused {
            block: block_ir::BlockId::from_raw(0),
            locals: vec![],
        }];
        assert!(is_still_running(&events));
    }

    #[test]
    fn finished_errored_or_waiting_are_not_still_running() {
        assert!(!is_still_running(&[RunEvent::Finished]));
        assert!(!is_still_running(&[RunEvent::Error("boom".to_string())]));
        assert!(!is_still_running(&[RunEvent::InputRequested]));
    }
}
