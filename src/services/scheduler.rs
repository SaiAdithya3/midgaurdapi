#[allow(unused_imports)]
use crate::utils::RUNEPOOL_START_TIME;
use chrono::Utc;
use cron::Schedule;
use log::{error, info};
use mongodb::Client;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
#[allow(unused_imports)]
use tokio::time::{interval, Duration};

// const INITIAL_START_TIME: i64 = 1707350400;
const INITIAL_START_TIME: i64 = 1739487600;
// static LAST_EXECUTION_TIME: AtomicI64 = AtomicI64::new(RUNEPOOL_START_TIME);
static LAST_EXECUTION_TIME: AtomicI64 = AtomicI64::new(INITIAL_START_TIME);

pub async fn start_hourly_data_fetch(mongo_client: Client) {
    // "0 0 * * * *" -> sec min hour day month weekday
    let schedule = match Schedule::from_str("0 0 * * * *") {
        Ok(schedule) => schedule,
        Err(e) => {
            error!("Failed to create schedule: {}", e);
            return;
        }
    };
    let last_execution = LAST_EXECUTION_TIME.load(Ordering::SeqCst);
    let mut upcoming = schedule.upcoming(Utc);

    loop {
        if let Some(datetime) = upcoming.next() {
            let current_time = Utc::now().timestamp();
            info!("Starting hourly data fetch at {}", current_time);

            let client1 = mongo_client.clone();
            let client2 = mongo_client.clone();
            let client3 = mongo_client.clone();
            let client4 = mongo_client.clone();
            let last_exec = last_execution;

            let fetch_tasks = tokio::join!(
                super::fetch_earnings_history::fetch_earnings_history("hour", last_exec, &client1),
                super::fetch_runepool_members_units_history::fetch_runepool_members_units_history(
                    "hour", last_exec, &client2
                ),
                super::fetch_depth_price_history::fetch_depth_price_history(
                    "BTC.BTC", "hour", last_exec, &client3
                ),
                super::fetch_swaps_history::fetch_swaps_history("hour", last_exec, &client4)
            );

            match fetch_tasks {
                (Ok(_), Ok(_), Ok(_), Ok(_)) => info!("All fetches completed successfully"),
                _ => error!("One or more fetches failed"),
            }

            LAST_EXECUTION_TIME.store(current_time, Ordering::SeqCst);
            info!("Completed hourly data fetch at {}", datetime);

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
