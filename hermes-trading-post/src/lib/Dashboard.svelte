<script lang="ts">
  import Chart from './Chart.svelte';
  import CollapsibleSidebar from './CollapsibleSidebar.svelte';
  import { onMount } from 'svelte';
  
  export let currentPrice: number = 0;
  export let connectionStatus: 'connected' | 'disconnected' | 'error' | 'loading' = 'loading';
  
  let sidebarCollapsed = false;
  
  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }
  
  let selectedGranularity = '5m';  // Candle size: 1m, 5m, 15m, 1h, 6h, 1D (Coinbase supported only)
  let selectedPeriod = '1H';        // Time range: 5Y, 1Y, 6M, 3M, 1M, 5D, 4H, 1H
  
  // Define valid granularities for each period - ONLY Coinbase supported values
  const validGranularities: Record<string, string[]> = {
    '1H': ['1m', '5m', '15m'],
    '4H': ['5m', '15m', '1h'],    // Removed 1m - too many candles (240)
    '5D': ['15m', '1h'],           // Removed 1m and 5m - too many candles  
    '1M': ['1h', '6h'],            // Removed 5m and 15m - too many candles
    '3M': ['1h', '6h', '1D'],      // Removed 15m - too many candles
    '6M': ['6h', '1D'],            // Removed 1h - too many candles
    '1Y': ['6h', '1D'],
    '5Y': ['1D']                   // Only daily for 5 years
  };
  
  function isGranularityValid(granularity: string, period: string): boolean {
    return validGranularities[period]?.includes(granularity) || false;
  }
  
  function selectGranularity(granularity: string) {
    if (isGranularityValid(granularity, selectedPeriod)) {
      selectedGranularity = granularity;
      console.log('Selected granularity:', granularity);
    }
  }
  
  function selectPeriod(period: string) {
    selectedPeriod = period;
    console.log('Selected period:', period);
    
    // If current granularity is not valid for new period, select the best default
    if (!isGranularityValid(selectedGranularity, period)) {
      const validOptions = validGranularities[period];
      if (validOptions && validOptions.length > 0) {
        // Select the middle option as default (usually the best balance)
        const middleIndex = Math.floor(validOptions.length / 2);
        selectedGranularity = validOptions[middleIndex];
        console.log('Auto-selected granularity:', selectedGranularity);
      }
    }
  }
</script>

<div class="dashboard-layout">
  <CollapsibleSidebar {sidebarCollapsed} on:toggle={toggleSidebar} />
  
  <main class="dashboard-content" class:expanded={sidebarCollapsed}>
    <!-- Header Bar -->
    <div class="header-bar">
      <h1>Trading Dashboard</h1>
      <div class="header-stats">
        <div class="stat-item">
          <span class="stat-label">BTC/USD</span>
          <span class="stat-value price">${currentPrice.toLocaleString('en-US', { minimumFractionDigits: 2 })}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Status</span>
          <span class="stat-value status {connectionStatus}">{connectionStatus}</span>
        </div>
      </div>
    </div>
    
    <!-- Dashboard Grid -->
    <div class="dashboard-grid">
      <!-- Main Chart Panel -->
      <div class="panel chart-panel">
        <div class="panel-header">
          <h2>BTC/USD Chart</h2>
          <div class="panel-controls">
            <div class="granularity-buttons">
              <button class="granularity-btn" class:active={selectedGranularity === '1m'} class:disabled={!isGranularityValid('1m', selectedPeriod)} on:click={() => selectGranularity('1m')}>1m</button>
              <button class="granularity-btn" class:active={selectedGranularity === '5m'} class:disabled={!isGranularityValid('5m', selectedPeriod)} on:click={() => selectGranularity('5m')}>5m</button>
              <button class="granularity-btn" class:active={selectedGranularity === '15m'} class:disabled={!isGranularityValid('15m', selectedPeriod)} on:click={() => selectGranularity('15m')}>15m</button>
              <button class="granularity-btn" class:active={selectedGranularity === '1h'} class:disabled={!isGranularityValid('1h', selectedPeriod)} on:click={() => selectGranularity('1h')}>1h</button>
              <button class="granularity-btn" class:active={selectedGranularity === '6h'} class:disabled={!isGranularityValid('6h', selectedPeriod)} on:click={() => selectGranularity('6h')}>6h</button>
              <button class="granularity-btn" class:active={selectedGranularity === '1D'} class:disabled={!isGranularityValid('1D', selectedPeriod)} on:click={() => selectGranularity('1D')}>1D</button>
            </div>
            <button class="panel-btn">⛶</button>
          </div>
        </div>
        <div class="panel-content">
          <Chart bind:status={connectionStatus} granularity={selectedGranularity} period={selectedPeriod} />
          <div class="period-buttons">
            <button class="period-btn" class:active={selectedPeriod === '1H'} on:click={() => selectPeriod('1H')}>1H</button>
            <button class="period-btn" class:active={selectedPeriod === '4H'} on:click={() => selectPeriod('4H')}>4H</button>
            <button class="period-btn" class:active={selectedPeriod === '5D'} on:click={() => selectPeriod('5D')}>5D</button>
            <button class="period-btn" class:active={selectedPeriod === '1M'} on:click={() => selectPeriod('1M')}>1M</button>
            <button class="period-btn" class:active={selectedPeriod === '3M'} on:click={() => selectPeriod('3M')}>3M</button>
            <button class="period-btn" class:active={selectedPeriod === '6M'} on:click={() => selectPeriod('6M')}>6M</button>
            <button class="period-btn" class:active={selectedPeriod === '1Y'} on:click={() => selectPeriod('1Y')}>1Y</button>
            <button class="period-btn" class:active={selectedPeriod === '5Y'} on:click={() => selectPeriod('5Y')}>5Y</button>
          </div>
        </div>
      </div>
      
      <!-- Bottom Panels Row -->
      <div class="bottom-panels">
        <!-- Portfolio Panel -->
        <div class="panel portfolio-panel">
          <div class="panel-header">
            <h2>Portfolio Summary</h2>
          </div>
          <div class="panel-content">
            <div class="portfolio-item">
              <span>BTC Balance:</span>
              <span>0.00000000</span>
            </div>
            <div class="portfolio-item">
              <span>USD Balance:</span>
              <span>$0.00</span>
            </div>
            <div class="portfolio-item">
              <span>Total Value:</span>
              <span>$0.00</span>
            </div>
          </div>
        </div>
        
        <!-- Market Stats Panel -->
        <div class="panel stats-panel">
          <div class="panel-header">
            <h2>Market Stats</h2>
          </div>
          <div class="panel-content">
            <div class="market-stat">
              <span>24h Volume:</span>
              <span>$2.4B</span>
            </div>
            <div class="market-stat">
              <span>24h High:</span>
              <span>$118,234</span>
            </div>
            <div class="market-stat">
              <span>24h Low:</span>
              <span>$116,892</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </main>
</div>

<style>
  .dashboard-layout {
    display: flex;
    height: 100vh;
    background: #0a0a0a;
    color: #d1d4dc;
  }
  
  .dashboard-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    margin-left: 250px;
    transition: margin-left 0.3s ease;
    overflow: hidden;
  }
  
  .dashboard-content.expanded {
    margin-left: 80px;
  }
  
  /* Header Bar */
  .header-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 30px;
    background: #0f0f0f;
    border-bottom: 1px solid rgba(74, 0, 224, 0.2);
  }
  
  .header-bar h1 {
    font-size: 24px;
    color: #a78bfa;
    margin: 0;
  }
  
  .header-stats {
    display: flex;
    gap: 30px;
  }
  
  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  
  .stat-label {
    font-size: 12px;
    color: #6b7280;
    text-transform: uppercase;
  }
  
  .stat-value {
    font-size: 18px;
    font-weight: bold;
  }
  
  .stat-value.price {
    color: #26a69a;
  }
  
  .stat-value.status {
    font-size: 14px;
  }
  
  .stat-value.connected {
    color: #10b981;
  }
  
  .stat-value.disconnected {
    color: #f59e0b;
  }
  
  .stat-value.error {
    color: #ef4444;
  }
  
  .stat-value.loading {
    color: #6b7280;
  }
  
  /* Dashboard Grid */
  .dashboard-grid {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    overflow: auto;
  }
  
  /* Panels */
  .panel {
    background: rgba(22, 33, 62, 0.3);
    border: 1px solid rgba(74, 0, 224, 0.3);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 1px solid rgba(74, 0, 224, 0.2);
  }
  
  .panel-header h2 {
    font-size: 16px;
    margin: 0;
    color: #a78bfa;
  }
  
  .panel-controls {
    display: flex;
    gap: 10px;
  }
  
  .panel-btn {
    background: none;
    border: 1px solid rgba(74, 0, 224, 0.3);
    color: #9ca3af;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }
  
  .panel-btn:hover {
    background: rgba(74, 0, 224, 0.2);
    color: #a78bfa;
  }
  
  .panel-content {
    flex: 1;
    padding: 20px;
    overflow: auto;
  }
  
  /* Specific Panel Layouts */
  .chart-panel {
    min-height: 400px;
    flex: 1;
    max-height: 600px;
  }
  
  .chart-panel .panel-content {
    padding: 0;
    height: calc(100% - 60px); /* Subtract header height */
    display: flex;
    flex-direction: column;
  }
  
  .chart-panel .panel-content > :global(.chart-container) {
    flex: 1;
  }
  
  .bottom-panels {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 20px;
    min-height: 200px;
  }
  
  .portfolio-panel, .stats-panel {
    height: 200px;
  }
  
  .portfolio-item, .market-stat {
    display: flex;
    justify-content: space-between;
    padding: 10px 0;
    border-bottom: 1px solid rgba(74, 0, 224, 0.1);
  }
  
  .portfolio-item:last-child, .market-stat:last-child {
    border-bottom: none;
  }
  
  /* Granularity buttons */
  .granularity-buttons {
    display: flex;
    gap: 2px;
  }
  
  .granularity-btn {
    background: rgba(74, 0, 224, 0.1);
    border: 1px solid rgba(74, 0, 224, 0.3);
    color: #9ca3af;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }
  
  .granularity-btn:hover {
    background: rgba(74, 0, 224, 0.2);
    color: #a78bfa;
  }
  
  .granularity-btn.active {
    background: rgba(74, 0, 224, 0.4);
    color: #e9d5ff;
    border-color: #a78bfa;
  }
  
  .granularity-btn.disabled {
    opacity: 0.3;
    cursor: not-allowed;
    pointer-events: none;
  }
  
  /* Period buttons */
  .period-buttons {
    display: flex;
    gap: 5px;
    justify-content: center;
    padding: 10px;
    background: rgba(0, 0, 0, 0.3);
    border-top: 1px solid rgba(74, 0, 224, 0.2);
  }
  
  .period-btn {
    background: rgba(74, 0, 224, 0.1);
    border: 1px solid rgba(74, 0, 224, 0.3);
    color: #9ca3af;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }
  
  .period-btn:hover {
    background: rgba(74, 0, 224, 0.2);
    color: #a78bfa;
  }
  
  .period-btn.active {
    background: rgba(74, 0, 224, 0.4);
    color: #e9d5ff;
    border-color: #a78bfa;
  }
</style>