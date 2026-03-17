from .financeindia import (
    FinanceClient,
    AsyncFinanceClient,
    FiiDiiActivity,
    MarketStatus,
    MarketStatusResponse,
    Holiday,
    ASMStock,
    GSMStock,
    EquityInfo,
    PriceVolumeRow,
    MarketStream,
    BhavArchive
)

try:
    import polars as pl
    HAS_POLARS = True
except ImportError:
    HAS_POLARS = False

def _create_df_wrapper(method_name, is_async=False):
    if is_async:
        async def async_wrapper(self, *args, **kwargs):
            if not HAS_POLARS:
                raise ImportError("polars is required for _df methods. Install with 'pip install polars'")
            method = getattr(self, method_name)
            data = await method(*args, **kwargs)
            return pl.from_records(data) if isinstance(data, list) else pl.from_dict(data)
        return async_wrapper
    else:
        def sync_wrapper(self, *args, **kwargs):
            if not HAS_POLARS:
                raise ImportError("polars is required for _df methods. Install with 'pip install polars'")
            method = getattr(self, method_name)
            data = method(*args, **kwargs)
            return pl.from_records(data) if isinstance(data, list) else pl.from_dict(data)
        return sync_wrapper

# List of methods that return tabular data suitable for Polars
TABULAR_METHODS = [
    "get_market_status", "get_fii_dii_activity", "price_volume_data",
    "deliverable_position_data", "bhav_copy_equities", "get_equity_list",
    "bulk_deal_data", "block_deals_data", "short_selling_data",
    "bhav_copy_derivatives", "get_span_margins", "get_fo_ban_list",
    "get_participant_volume", "get_insider_trades", "get_slb_bhavcopy",
    "get_currency_bhavcopy", "get_nse_commodities_bhavcopy"
]

for method in TABULAR_METHODS:
    if hasattr(FinanceClient, method):
        setattr(FinanceClient, f"{method}_df", _create_df_wrapper(method, is_async=False))
    if hasattr(AsyncFinanceClient, method):
        setattr(AsyncFinanceClient, f"{method}_df", _create_df_wrapper(method, is_async=True))

__all__ = [
    "ASMStock",
    "AsyncFinanceClient",
    "BhavArchive",
    "EquityInfo",
    "FiiDiiActivity",
    "FinanceClient",
    "GSMStock",
    "Holiday",
    "MarketStatus",
    "MarketStatusResponse",
    "MarketStream",
    "PriceVolumeRow"
]
