use std::sync::{Arc, OnceLock};

use fsrs::FSRSError;
use itertools::Itertools;
use parking_lot::Mutex;

use crate::tracking::{tracking, CardInfo, CardReview};

static FSRS: OnceLock<Mutex<fsrs::FSRS>> = OnceLock::new();

/// Initialize FSRS struct with params, must be called once before any other fsrs stuff.
pub fn init_fsrs(params: &[f32]) {
    FSRS.get_or_init(|| {
        Mutex::new(fsrs::FSRS::new(Some(params)).expect("Couldn't initialize FSRS"))
    });
}

fn get_fsrs() -> &'static Mutex<fsrs::FSRS> {
    FSRS.get().expect("FSRS not initialized")
}

/// Set the params for the FSRS struct
pub fn set_params(params: &[f32]) {
    *get_fsrs().lock() = fsrs::FSRS::new(Some(params)).expect("Couldn't reinitialize FSRS");
}

pub fn update_card(
    card: &mut CardInfo,
    review: CardReview,
    retention: f32,
) -> Result<(), FSRSError> {
    let ctx = get_fsrs().lock();
    let last_review = card
        .reviews
        .last()
        .map(|rev| rev.timestamp)
        .unwrap_or(review.timestamp);
    let elapsed = ((review.timestamp - last_review) / (24 * 60 * 60)) as u32;
    let next_states = ctx.next_states(card.memory_state.map(|s| s.into()), retention, elapsed)?;

    let state = match review.rating {
        crate::cards::Rating::Again => next_states.again,
        crate::cards::Rating::Hard => next_states.hard,
        crate::cards::Rating::Good => next_states.good,
        crate::cards::Rating::Easy => next_states.easy,
    };
    let interval = (state.interval * (24.0 * 60.0 * 60.0)) as i64;

    card.memory_state = Some(state.memory.into());
    card.due = Some(review.timestamp + interval);
    card.reviews.push(review);

    Ok(())
}

/// Compute FSRS params from card history, this can take a long time.
pub fn update_params(
    progress: Option<Arc<std::sync::Mutex<fsrs::CombinedProgressState>>>,
) -> Result<(), FSRSError> {
    let mut tracking = tracking().lock();
    let num_revs: usize = tracking.cards_info.values().map(|c| c.reviews.len()).sum();
    if num_revs < 400 {
        return Err(FSRSError::NotEnoughData);
    }

    let new_params = {
        let ctx = get_fsrs().lock();
        let train_set = tracking.cards_info.values().map_into().collect_vec();
        ctx.compute_parameters(train_set, progress, false)?
    };
    tracking.update_params(new_params);
    tracking.save();
    Ok(())
}
