use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Midgard History API",
        version = "1.0.0",
        description = "THORChain Midgard Historical Data API Service",
    ),
    paths(
        crate::routes::depth_history_routes::get_depth_history,
        crate::routes::rune_pool_history_route::get_runepool_history,
        crate::routes::swaps_history_routes::get_swaps_history,
        crate::routes::earning_history_route::get_earnings_history
    ),
    components(
        schemas(
            crate::routes::queries::HistoryQueryParams,
            crate::models::depth_price_history::DepthPriceHistory,
            crate::models::swaps_history::SwapsHistory,
            crate::models::runepool_members_units_history::RunePoolTotalMembersHistory,
            crate::models::earnings_history::EarningsHistory
        )
    ),
    tags(
        (name = "Depth History", description = "Pool depth and price history endpoints"),
        (name = "Rune Pool History", description = "RUNE pool statistics and metrics"),
        (name = "Swaps History", description = "Historical swap data and analytics"),
        (name = "Earnings History", description = "Historical earnings and rewards data")
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server")
    )
)]
pub struct ApiDoc;
