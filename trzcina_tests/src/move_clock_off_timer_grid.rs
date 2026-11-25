use tokio::time::advance;

use crate::smallest_duration_step::SMALLEST_DURATION_STEP;

pub async fn move_clock_off_timer_grid() {
    advance(SMALLEST_DURATION_STEP).await;
}
