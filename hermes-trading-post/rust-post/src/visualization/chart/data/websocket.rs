use super::{Candle, CandleUpdate};
use tokio::sync::mpsc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tungstenite::Message;
use reqwest;
use tokio::sync::watch;

pub async fn fetch_historical_candles() -> anyhow::Result<Vec<Candle>> {
    // Use Coinbase Exchange API (public endpoint)
    // Granularity: 60 = 1 minute candles
    let url = "https://api.exchange.coinbase.com/products/BTC-USD/candles?granularity=60";
    let client = reqwest::Client::new();
    let resp = client.get(url)
        .header("User-Agent", "rust-trading-app/1.0")
        .send()
        .await?
        .text()
        .await?;
    
    // Coinbase Exchange returns: [[time, low, high, open, close, volume], ...]
    let json: serde_json::Value = serde_json::from_str(&resp)?;
    let mut candles = Vec::new();
    if let Some(arr) = json.as_array() {
        for entry in arr {
            if let Some(vals) = entry.as_array() {
                if vals.len() >= 6 {
                    candles.push(Candle {
                        open: vals[3].as_f64().unwrap_or(0.0),
                        high: vals[2].as_f64().unwrap_or(0.0),
                        low: vals[1].as_f64().unwrap_or(0.0),
                        close: vals[4].as_f64().unwrap_or(0.0),
                        _volume: vals[5].as_f64().unwrap_or(0.0),
                    });
                }
            }
        }
    }
    // Reverse to get chronological order (API returns newest first)
    candles.reverse();
    
    // Remove any duplicate candles (by similar prices)
    let original_count = candles.len();
    candles.dedup_by(|a, b| {
        (a.open - b.open).abs() < 0.01 && 
        (a.close - b.close).abs() < 0.01 &&
        (a.high - b.high).abs() < 0.01 &&
        (a.low - b.low).abs() < 0.01
    });
    
    if candles.len() < original_count {
        log::warn!("⚠️ Removed {} duplicate candles", original_count - candles.len());
    }
    
    // Validate candle data
    candles.retain(|candle| {
        let valid = candle.high >= candle.low && 
                   candle.high >= candle.open && 
                   candle.high >= candle.close &&
                   candle.low <= candle.open &&
                   candle.low <= candle.close &&
                   candle.open > 0.0 &&
                   candle.close > 0.0;
        if !valid {
            log::warn!("⚠️ Skipping invalid candle: O:{} H:{} L:{} C:{}", 
                      candle.open, candle.high, candle.low, candle.close);
        }
        valid
    });
    
    log::info!("📊 Fetched {} valid candles from Coinbase API", candles.len());
    Ok(candles)
}

pub async fn start_websocket_with_reconnect(tx: mpsc::Sender<CandleUpdate>, mut shutdown_rx: watch::Receiver<bool>) {
    loop {
        if *shutdown_rx.borrow() {
            log::info!("🛑 WebSocket shutdown requested, exiting...");
            break;
        }
        
        log::info!("🔌 Starting WebSocket connection...");
        if let Err(e) = websocket_task(tx.clone(), shutdown_rx.clone()).await {
            if *shutdown_rx.borrow() {
                log::info!("🛑 WebSocket shutdown during error recovery");
                break;
            }
            log::error!("❌ WebSocket error: {}. Reconnecting in 5 seconds...", e);
            
            // Use select to allow cancellation during sleep
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {}
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        log::info!("🛑 WebSocket shutdown during reconnect delay");
                        break;
                    }
                }
            }
        }
    }
}

async fn websocket_task(tx: mpsc::Sender<CandleUpdate>, mut shutdown_rx: watch::Receiver<bool>) -> anyhow::Result<()> {
    use tokio_tungstenite::connect_async;

    let url = "wss://ws-feed.exchange.coinbase.com";
    
    log::info!("🌐 Connecting to Coinbase WebSocket: {}", url);
    
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // Connection lifetime tracking
    let connection_start = std::time::Instant::now();
    let max_connection_lifetime = std::time::Duration::from_secs(3600); // Reconnect every hour

    // Subscribe to ticker for real-time updates
    let subscribe_msg = serde_json::json!({
        "type": "subscribe",
        "channels": [{
            "name": "ticker",
            "product_ids": ["BTC-USD"]
        }]
    });
    
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    log::info!("✅ Subscribed to BTC-USD ticker");

    // Track candle data
    let mut current_candle: Option<Candle> = None;
    let mut candle_start_time = std::time::Instant::now();
    let candle_duration = std::time::Duration::from_secs(60); // 1 minute candles
    
    // Heartbeat interval for keeping connection alive
    let heartbeat_interval = std::time::Duration::from_secs(30);
    let mut last_heartbeat = std::time::Instant::now();

    loop {
        // Check if connection has exceeded maximum lifetime
        if connection_start.elapsed() > max_connection_lifetime {
            log::info!("⏰ Connection lifetime exceeded, initiating graceful reconnection");
            break;
        }
        
        tokio::select! {
            msg = read.next() => {
                match msg {
                    None => break,
                    Some(Ok(Message::Text(txt))) => {
                if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                    if json["type"] == "ticker" && json["product_id"] == "BTC-USD" {
                        if let Some(price_str) = json["price"].as_str() {
                            if let Ok(price) = price_str.parse::<f64>() {
                                process_price_update(price, &mut current_candle, &mut candle_start_time, candle_duration, &tx).await;
                            }
                        }
                    }
                }
            }
                    Some(Ok(Message::Close(_))) => {
                        log::warn!("🔌 WebSocket connection closed by server");
                        break;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        log::debug!("🏓 Received pong");
                    }
                    Some(Err(e)) => {
                        log::error!("❌ WebSocket message error: {}", e);
                        break;
                    }
                    Some(_) => {}
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                // Check if we need to send a heartbeat ping
                if last_heartbeat.elapsed() > heartbeat_interval {
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        log::error!("❌ Failed to send ping: {}", e);
                        break;
                    }
                    log::debug!("🏓 Sent ping");
                    last_heartbeat = std::time::Instant::now();
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    log::info!("🛑 WebSocket shutdown requested");
                    break;
                }
            }
        }
    }
    
    Ok(())
}

async fn process_price_update(
    price: f64,
    current_candle: &mut Option<Candle>,
    candle_start_time: &mut std::time::Instant,
    candle_duration: std::time::Duration,
    tx: &mpsc::Sender<CandleUpdate>,
) {
    match current_candle {
        None => {
            // First price - create initial candle
            *current_candle = Some(Candle {
                open: price,
                high: price,
                low: price,
                close: price,
                _volume: 0.0,
            });
            *candle_start_time = std::time::Instant::now();
            log::info!("🕯️ Created initial candle at ${:.2}", price);
        }
        Some(candle) => {
            // Check if we need a new candle
            if candle_start_time.elapsed() >= candle_duration {
                // Send the completed candle
                let _ = tx.send(CandleUpdate::NewMinuteCandle(candle.clone())).await;
                log::info!("✅ Completed candle: O:{:.2} H:{:.2} L:{:.2} C:{:.2}", 
                    candle.open, candle.high, candle.low, candle.close);
                
                // Start new candle
                *candle = Candle {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    _volume: 0.0,
                };
                *candle_start_time = std::time::Instant::now();
            } else {
                // Update current candle - keep open price constant
                candle.close = price;
                candle.high = candle.high.max(price);
                candle.low = candle.low.min(price);
            }
        }
    }
    
    // Send live update
    if let Some(candle) = current_candle {
        let _ = tx.send(CandleUpdate::LiveUpdate(candle.clone())).await;
    }
}