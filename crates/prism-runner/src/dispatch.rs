use crate::Route;
use prism_runtime::b1::{FilterId, ObservationOutcome, Observer, Pipeline};
use prism_runtime::Outcome;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ObservationStats {
    pub events: usize,
    pub completed: usize,
    pub rejected: usize,
    pub execution_failures: usize,
}

impl Observer for ObservationStats {
    fn on_filter(&mut self, _filter: FilterId, _elapsed_ns: u128, outcome: ObservationOutcome) {
        self.events += 1;
        match outcome {
            ObservationOutcome::Completed => self.completed += 1,
            ObservationOutcome::Rejected => self.rejected += 1,
            ObservationOutcome::ExecutionFailure => self.execution_failures += 1,
        }
    }
}

pub fn run(route: Route, payload: &[u8]) -> Result<(Outcome, ObservationStats), String> {
    match route {
        Route::B0 => Ok((
            prism_runtime::b0::process(payload),
            ObservationStats::default(),
        )),
        Route::B1 => Ok((
            Pipeline::new()
                .map_err(|_| "pipeline registry failure".to_owned())?
                .process(payload),
            ObservationStats::default(),
        )),
        Route::B2 => {
            let pipeline = Pipeline::new().map_err(|_| "pipeline registry failure".to_owned())?;
            let mut stats = ObservationStats::default();
            let outcome = pipeline.process_observed(payload, &mut stats);
            Ok((outcome, stats))
        }
    }
}
