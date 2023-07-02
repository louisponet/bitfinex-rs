use ticker::*;
use candles::Candle;
use trades::{TradingPair as TradesTradingPair, FundingCurrency as TradesFundingCurrency};
use book::{TradingPair as BookTradingPair, FundingCurrency as BookFundingCurrency, RawBook};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum NotificationEvent {
    Auth(AuthMessage),
    Info(InfoMessage),
    TradingSubscribed(TradingSubscriptionMessage),
    FundingSubscribed(FundingSubscriptionMessage),
    CandlesSubscribed(CandlesSubscriptionMessage),
    RawBookSubscribed(RawBookSubscriptionMessage),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DataEvent {
    TickerTradingEvent (i32, TradingPair),
    TickerFundingEvent (i32, FundingCurrency),
    TradesTradingSnapshotEvent (i32, Vec<TradesTradingPair>),
    TradesTradingUpdateEvent (i32, String, TradesTradingPair),
    TradesFundingSnapshotEvent (i32, Vec<TradesFundingCurrency>),
    TradesFundingUpdateEvent (i32, String, TradesFundingCurrency),
    BookTradingSnapshotEvent (i32, Vec<BookTradingPair>),
    BookTradingUpdateEvent (i32, BookTradingPair),
    BookFundingSnapshotEvent (i32, Vec<BookFundingCurrency>),
    BookFundingUpdateEvent (i32, BookFundingCurrency),
    RawBookEvent (i32, RawBook),
    RawBookUpdateEvent (i32, Vec<RawBook>),
    CandlesSnapshotEvent (i32, Vec<Candle>),
    CandlesUpdateEvent (i32, Candle),
    HeartbeatEvent (i32, String),
    PositionSnapshotEvent (i32, String, Vec<PositionUpdate>),
    WalletsSnapshotEvent (i32, String, Vec<WalletUpdate>),
    WalletsUpdateEvent (i32, String, WalletUpdate),
    BalanceUpdateEvent( i32, String, BalanceUpdate)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMessage {
    pub event: String,
    pub status: String,
    pub chan_id: u32,
    pub code: Option<u32>,
    pub msg: Option<String>,
    pub user_id: Option<u32>,
    pub auth_id: Option<String>,
}

impl AuthMessage {
    pub fn is_ok(&self) -> bool {
        self.status == "OK"
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoMessage {
    pub event: String,
    pub version: u16,
    pub server_id: String,
    pub platform: Platform,
}

#[derive(Debug, Deserialize)]
pub struct Platform {
    pub status: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub pair: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub currency: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub key: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawBookSubscriptionMessage {
    pub event: String,
    pub channel: String,
    pub chan_id: u32,
    pub symbol: String,
    pub prec: String,
    pub freq: String,
    pub len: String,
    pub pair: String

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionUpdate {
    pub symbol: String,
    pub status: String,
    pub diff: f64,
    pub base_price: f64,
    pub margin_funding: f64,
    pub margin_funding_type: u64,
    pub pl: f64,
    pub pl_perc: f64,
    pub price_liq: f64,
    pub position_id: u64,
    pub mts_create: u64,
    pub mts_update: u64,
    pub collateral: f64,
    pub collateral_min: f64,
    pub meta: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletUpdate {
    pub wallet_type: String,
    pub currency: String,
    pub balance: f64,
    pub unsettled_interest: f64,
    pub balance_available: Option<f64>,
    pub description: Option<String>,
    pub meta: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeUpdate {
    pub id: u64,
    pub cid: u64,
    pub symbol: String,
    pub mts_create: u64,
    pub order_id: u64,
    pub exec_amount: f64,
    pub exec_price: f64,
    pub order_type: String,
    pub order_price: f64,
    pub maker: Option<bool>,
    pub fee: Option<f64>,
    pub fee_currency: Option<String>
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceUpdate {
    pub aum: f64,
    pub aum_net: f64,
}


