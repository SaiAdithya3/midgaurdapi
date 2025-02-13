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
    "asset_depth": f64,
    "rune_depth": f64,
    "asset_price": f64,
    "asset_price_usd": f64,
    "start_time": i64,
    "end_time": i64
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
    "earnings": f64,
    "block_rewards": f64,
    "liquidity_fees": f64,
    "start_time": i64,
    "end_time": i64
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
- sort: String (Optional) - Sort by [start_time, end_time, asset_depth, rune_depth, units]
- order: String (Optional) - Order [asc, desc]

```json
Response: {
    "asset": String,
    "asset_depth": f64,
    "rune_depth": f64,
    "units": f64,
    "start_time": i64,
    "end_time": i64
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
    "volume": f64,
    "average_slip": f64,
    "fees": f64,
    "start_time": i64,
    "end_time": i64
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