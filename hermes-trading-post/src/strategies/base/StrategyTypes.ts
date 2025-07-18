// Common types for all trading strategies

// Re-export CandleData from the existing definition
export type { CandleData } from '../../types/coinbase';

export interface Position {
  entryPrice: number;
  entryTime: number;
  size: number;
  type: 'long' | 'short';
  stopLoss?: number;
  takeProfit?: number;
  metadata?: {
    level?: number;  // For reverse ratio strategy
    reason?: string;
    [key: string]: any;
  };
}

export interface Signal {
  type: 'buy' | 'sell' | 'hold';
  strength: number; // 0-1
  price: number;
  size?: number;
  reason: string;
  metadata?: {
    level?: number;
    targetPrice?: number;
    [key: string]: any;
  };
}

export interface Trade {
  id: string;
  timestamp: number;
  type: 'buy' | 'sell';
  price: number;
  size: number;
  value: number;
  fee?: number;
  grossFee?: number;    // Total fee before rebates
  feeRebate?: number;   // Fee rebate amount
  position?: Position;
  profit?: number;
  profitPercent?: number;
  reason: string;
}

export interface StrategyConfig {
  // Common configuration
  vaultAllocation: number;      // % of profits to vault (0-100)
  btcGrowthAllocation: number;  // % of profits to keep in BTC (0-100)
  maxDrawdown?: number;         // Optional max drawdown %
  
  // Strategy-specific parameters
  [key: string]: any;
}

export interface BacktestResult {
  trades: Trade[];
  metrics: {
    totalTrades: number;
    winningTrades: number;
    losingTrades: number;
    winRate: number;
    totalReturn: number;
    totalReturnPercent: number;
    maxDrawdown: number;
    maxDrawdownPercent: number;
    sharpeRatio: number;
    profitFactor: number;
    averageWin: number;
    averageLoss: number;
    averageHoldTime: number; // in hours
    vaultBalance: number;
    btcGrowth: number;
    // New metrics
    avgPositionSize: number;
    tradesPerDay: number;
    tradesPerWeek: number;
    tradesPerMonth: number;
    totalFees: number;
    feesAsPercentOfProfit: number;
    vaultCAGR: number; // Compound Annual Growth Rate
    btcGrowthPercent: number;
    maxConsecutiveLosses: number;
    riskRewardRatio: number;
    // Balance growth metrics
    initialBalanceGrowth: number;        // USD growth from profit reinvestment
    initialBalanceGrowthPercent: number; // Percentage growth of initial balance
    finalTradingBalance: number;         // Final USD available for trading
    totalFeeRebates: number;             // Total fee rebates received
    netFeesAfterRebates: number;         // Net fees paid after rebates
  };
  equity: Array<{
    timestamp: number;
    value: number;
    btcBalance: number;
    usdBalance: number;
    vaultBalance: number;
  }>;
  // Time series data for charts
  chartData: {
    vaultGrowth: Array<{time: number; value: number}>;
    btcGrowth: Array<{time: number; value: number}>;
    equityCurve: Array<{time: number; value: number}>;
    drawdown: Array<{time: number; value: number}>;
    tradeDistribution: {
      daily: Map<string, number>;
      weekly: Map<string, number>;
      monthly: Map<string, number>;
    };
  };
}

export interface StrategyState {
  positions: Position[];
  balance: {
    usd: number;
    btcVault: number;    // BTC accumulated from profit allocations
    btcPositions: number; // BTC currently held in active positions
    vault: number;       // USDC vault balance
  };
  lastSignal?: Signal;
  metadata?: {
    [key: string]: any;
  };
}

