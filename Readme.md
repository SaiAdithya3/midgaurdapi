# Midgard Historical Data Service

A Rust-based service that synchronizes and serves historical data from THORChain's Midgard API.

## API Routes

### 1. Base Routes
``` code
GET /                  - Welcome message
```

``` code
GET /health           - Service health check
``` 

### 2. Data Routes

#### Depth History

``` typescript
GET /api/history/depth/{pool}
```
Parameters:

- pool: String (Required) - Pool identifier (e.g., "BTC.BTC")
- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
- page: i32 (Optional) - Page number
- limit: i32 (Optional) - Limit per page
- sort: String (Optional) - Sort by [start_time, end_time, asset_depth, rune_depth, asset_price, asset_price_usd]
- order: String (Optional) - Order [asc, desc]
```json
Response:
{
    "pool": String,
    "start_time": i64,
    "end_time": i64,
    "asset_depth": f64,
    "rune_depth": f64,
    "asset_price": f64,
    "asset_price_usd": f64,
    "liquidity_units": f64,
    "members_count": f64,
    "synth_units": f64,
    "synth_supply": f64,
    "units": f64,
    "luvi": f64,
}
```

#### Earnings History

``` typescript
GET /api/history/earnings
```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
- page: i32 (Optional) - Page number
- limit: i32 (Optional) - Limit per page
- sort: String (Optional) - Sort by [start_time, end_time, earnings, block_rewards, liquidity_fees]
- order: String (Optional) - Order [asc, desc]

```json
Response: {
    "start_time": i64,
    "end_time": i64,
    "block_rewards": f64,
    "avg_node_count": f64,
    "bonding_earnings": f64,
    "liquidity_earnings": f64,
    "liquidity_fees": f64,
    "rune_price_usd": f64,
    "pools": [
        {
            "_id": ObjectId,
            "pool": String,
            "asset_liquidity_fees": f64,
            "rune_liquidity_fees": f64,
            "total_liquidity_fees_rune": f64,
            "saver_earning": f64,
            "rewards": f64,
            "start_time": i64,
            "end_time": i64,
        }
    ]
}
```

#### Runepool History

``` code
GET /api/history/runepool
 ```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
- page: i32 (Optional) - Page number
- limit: i32 (Optional) - Limit per page
- sort: String (Optional) - Sort by [start_time, end_time, depth, rune_depth, units]
- order: String (Optional) - Order [asc, desc]

```json
Response: {
    "asset": String,
    "depth": f64,
    "rune_depth": f64,
    "units": f64,
    "start_time": i64,
    "end_time": i64,
    "count": f64,
}
```

#### Swaps History

``` code
GET /api/history/swaps
 ```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
- page: i32 (Optional) - Page number
- limit: i32 (Optional) - Limit per page
- sort: String (Optional) - Sort by [start_time, end_time, volume, average_slip, fees]
- order: String (Optional) - Order [asc, desc]
```json
Response: {
    "pool": "String",
    "start_time": "i64",
    "end_time": "i64",
    "to_asset_count": "i64",
    "to_rune_count": "i64", 
    "to_trade_count": "i64",
    "from_trade_count": "i64",
    "synth_mint_count": "i64",
    "synth_redeem_count": "i64",
    "total_count": "i64",
    "to_asset_volume": "f64",
    "to_rune_volume": "f64",
    "to_trade_volume": "f64", 
    "from_trade_volume": "f64",
    "synth_mint_volume": "f64",
    "synth_redeem_volume": "f64",
    "total_volume": "f64",
    "to_asset_volume_usd": "f64",
    "to_rune_volume_usd": "f64",
    "to_trade_volume_usd": "f64",
    "from_trade_volume_usd": "f64",
    "synth_mint_volume_usd": "f64",
    "synth_redeem_volume_usd": "f64",
    "total_volume_usd": "f64",
    "to_asset_fees": "f64",
    "to_rune_fees": "f64",
    "to_trade_fees": "f64",
    "from_trade_fees": "f64",
    "synth_mint_fees": "f64",
    "synth_redeem_fees": "f64",
    "total_fees": "f64",
    "to_asset_average_slip": "f64",
    "to_rune_average_slip": "f64",
    "to_trade_average_slip": "f64",
    "from_trade_average_slip": "f64",
    "synth_mint_average_slip": "f64",
    "synth_redeem_average_slip": "f64",
    "average_slip": "f64",
    "rune_price_usd": "f64"
}
```


## MongoDB Collections

### 1. depth_history
### 2. earnings_history 
### 3. runepool_history
### 4. swaps_history


## Background Services
### Automated Data Synchronization
- Hourly data fetching from Midgard API
- Concurrent fetching for all data types
- Atomic timestamp tracking
- Error handling and logging
## Error Responses
```json
{
    "error": "Error message description"
}
 ```

Common error cases:

- Invalid interval format
- Count out of range (1-400)
- Missing required parameters
- Database errors
- Invalid pool identifier
## Setup & Running
1. Environment Requirements:
   - Rust
   - MongoDB
   - Cargo
2. Clone the repository:
```bash
git clone https://github.com/SaiAdithya3/midgaurd.git
```
3. Start the service:
```bash
cargo run
 ```


Service will be available at http://127.0.0.1:8080