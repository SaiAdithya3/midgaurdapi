static POOL: &str = "BTC.BTC";
static apiurl: &str = "https://midgard.ninerealms.com/v2/history/depths/BTC.BTC/?interval=day&count=400&from=1606780800";

pub struct Meta {
    pub start_time: String,
    pub end_time: String,
    pub price_shift_loss: String,
    pub luvi_increase: String,
    pub start_asset_depth: String,
    pub start_rune_depth: String,
    pub start_lp_units: String,
    pub start_member_count: String,
    pub start_synth_units: String,
    pub end_asset_depth: String,
    pub end_rune_depth: String,
    pub end_lp_units: String,
    pub end_member_count: String,
    pub end_synth_units: String,
}

pub struct Interval {
    pub asset_depth: String,
    pub asset_price: String,
    pub asset_price_usd: String,
    pub end_time: String,
    pub liquidity_units: String,
    pub luvi: String,
    pub members_count: String,
    pub rune_depth: String,
    pub start_time: String,
    pub synth_supply: String,
    pub synth_units: String,
    pub units: String,
}

async fn fetch () -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.get(apiurl).send().await?;
    let body = res.text().await?;
    println!("{}", body);
    Ok(())
}
