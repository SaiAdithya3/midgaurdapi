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

### 2. Historical Data Routes

#### Depth History

``` code
GET /api/history/depth/{pool}
```
Parameters:

- pool: String (Required) - Pool identifier (e.g., "BTC.BTC")
- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
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

#### Earnings History

``` code
GET /api/history/earnings
```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
Response:
{
"earnings": f64,
"block_rewards": f64,
"liquidity_fees": f64,
"start_time": i64,
"end_time": i64
}

#### Runepool History

``` code
GET /api/history/runepool
 ```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
Response:
{
"asset": String,
"asset_depth": f64,
"rune_depth": f64,
"units": f64,
"start_time": i64,
"end_time": i64
}

#### Swaps History

``` code
GET /api/history/swaps
 ```

Parameters:

- interval: String (Optional) - Time interval [5min, hour, day, week, month, quarter, year]
- from: i64 (Optional) - Start timestamp
- to: i64 (Optional) - End timestamp
- count: i32 (Optional) - Number of records (1-400)
Response:
{
"volume": f64,
"average_slip": f64,
"fees": f64,
"start_time": i64,
"end_time": i64
}

```plaintext

## MongoDB Collections

### 1. depth_history
Stores historical depth and price data for pools
```typescript
{
    _id: ObjectId,
    pool: String,
    asset_depth: Double,
    rune_depth: Double,
    asset_price: Double,
    asset_price_usd: Double,
    start_time: Int64,
    end_time: Int64
}
 ```

### 2. earnings_history
Stores earnings data across time periods

```typescript
{
    _id: ObjectId,
    earnings: Double,
    block_rewards: Double,
    liquidity_fees: Double,
    start_time: Int64,
    end_time: Int64
}
 ```

### 3. runepool_history
Stores RUNE pool historical data

```typescript
{
    _id: ObjectId,
    asset: String,
    asset_depth: Double,
    rune_depth: Double,
    units: Double,
    start_time: Int64,
    end_time: Int64
}
 ```

### 4. swaps_history
Stores historical swap data

```typescript
{
    _id: ObjectId,
    volume: Double,
    average_slip: Double,
    fees: Double,
    start_time: Int64,
    end_time: Int64
}
 ```

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
2. Start the service:
```bash
cargo run
 ```

Service will be available at http://127.0.0.1:8080