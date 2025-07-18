<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createChart, type IChartApi, type ISeriesApi } from 'lightweight-charts';
  import { chartStore } from '../../stores/chartStore.svelte';
  import { dataStore } from '../../stores/dataStore.svelte';
  import { statusStore } from '../../stores/statusStore.svelte';
  import { performanceStore } from '../../stores/performanceStore.svelte';
  import { CHART_COLORS } from '../../utils/constants';
  
  export let width: number | undefined = undefined;
  export let height: number | undefined = undefined;
  export let onChartReady: ((chart: IChartApi) => void) | undefined = undefined;
  
  let container: HTMLDivElement;
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<'Candlestick'> | null = null;
  let resizeObserver: ResizeObserver | null = null;
  
  // Reactive chart options based on store
  $: chartOptions = {
    layout: {
      background: { 
        color: CHART_COLORS[chartStore.config.theme.toUpperCase()].background 
      },
      textColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].text,
    },
    grid: {
      vertLines: { 
        visible: chartStore.config.showGrid,
        color: CHART_COLORS[chartStore.config.theme.toUpperCase()].grid 
      },
      horzLines: { 
        visible: chartStore.config.showGrid,
        color: CHART_COLORS[chartStore.config.theme.toUpperCase()].grid 
      },
    },
    crosshair: {
      mode: chartStore.config.showCrosshair ? 1 : 0,
      vertLine: {
        width: 1,
        color: CHART_COLORS[chartStore.config.theme.toUpperCase()].text + '40',
        style: 2,
      },
      horzLine: {
        width: 1,
        color: CHART_COLORS[chartStore.config.theme.toUpperCase()].text + '40',
        style: 2,
      },
    },
    timeScale: {
      timeVisible: true,
      secondsVisible: false,
      borderColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].border,
      fixLeftEdge: true,
      fixRightEdge: true,
    },
    rightPriceScale: {
      borderColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].border,
      autoScale: true,
    },
    watermark: {
      visible: false,
    },
    handleScroll: {
      mouseWheel: true,
      pressedMouseMove: true,
      horzTouchDrag: true,
      vertTouchDrag: false,
    },
    handleScale: {
      axisPressedMouseMove: true,
      mouseWheel: true,
      pinch: true,
    },
  };
  
  onMount(() => {
    initializeChart();
    
    return () => {
      cleanup();
    };
  });
  
  function initializeChart() {
    if (!container) return;
    
    const startTime = performance.now();
    
    // Create chart
    chart = createChart(container, {
      ...chartOptions,
      width: width || container.clientWidth,
      height: height || container.clientHeight,
    });
    
    // Create candlestick series
    candleSeries = chart.addCandlestickSeries({
      upColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].upColor,
      downColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].downColor,
      borderVisible: false,
      wickUpColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].upColor,
      wickDownColor: CHART_COLORS[chartStore.config.theme.toUpperCase()].downColor,
    });
    
    // Store chart instance
    chartStore.setChartInstance({
      api: chart,
      series: candleSeries,
      config: chartStore.config
    });
    
    // Set up resize observer
    if (!width || !height) {
      resizeObserver = new ResizeObserver((entries) => {
        if (chart && entries.length > 0) {
          const { width: newWidth, height: newHeight } = entries[0].contentRect;
          chart.applyOptions({ width: newWidth, height: newHeight });
        }
      });
      resizeObserver.observe(container);
    }
    
    // Subscribe to visible range changes
    chart.timeScale().subscribeVisibleTimeRangeChange(() => {
      const visibleRange = chart!.timeScale().getVisibleRange();
      if (visibleRange) {
        // Debug visible range for 1d granularity
        if (chartStore.config.granularity === '1d') {
          console.log('[ChartCanvas] Visible range changed for 1d:', {
            from: new Date(Number(visibleRange.from) * 1000).toISOString(),
            to: new Date(Number(visibleRange.to) * 1000).toISOString(),
            rangeInDays: (Number(visibleRange.to) - Number(visibleRange.from)) / 86400
          });
        }
        
        dataStore.updateVisibleRange(
          Number(visibleRange.from),
          Number(visibleRange.to)
        );
      }
    });
    
    // Subscribe to crosshair move
    chart.subscribeCrosshairMove((param) => {
      if (param.time && param.seriesData.size > 0) {
        // Could emit crosshair data if needed
      }
    });
    
    // Initial data render
    if (dataStore.candles.length > 0) {
      updateChartData();
    }
    
    // Record initialization time
    performanceStore.recordRenderTime(performance.now() - startTime);
    
    // Notify that chart is ready
    if (onChartReady) {
      onChartReady(chart);
    }
  }
  
  function updateChartData() {
    if (!candleSeries || dataStore.isEmpty) return;
    
    const startTime = performance.now();
    
    try {
      candleSeries.setData(dataStore.candles);
      performanceStore.recordRenderTime(performance.now() - startTime);
    } catch (error) {
      console.error('ChartCanvas: Error updating data:', error);
      statusStore.setError('Failed to update chart data');
    }
  }
  
  function cleanup() {
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    
    if (chart) {
      chart.remove();
      chart = null;
    }
    
    candleSeries = null;
  }
  
  // Handle real-time updates
  function handleRealtimeUpdate(candle: any) {
    if (!candleSeries) return;
    
    const startTime = performance.now();
    
    try {
      candleSeries.update(candle);
      performanceStore.recordRenderTime(performance.now() - startTime);
      performanceStore.recordCandleUpdate();
    } catch (error) {
      console.error('ChartCanvas: Error updating candle:', error);
    }
  }
  
  // Reactive updates
  $: if (chart && chartOptions) {
    chart.applyOptions(chartOptions);
  }
  
  $: if (candleSeries && chartStore.config) {
    const colors = CHART_COLORS[chartStore.config.theme.toUpperCase()];
    candleSeries.applyOptions({
      upColor: colors.upColor,
      downColor: colors.downColor,
      wickUpColor: colors.upColor,
      wickDownColor: colors.downColor,
    });
  }
  
  // Data updates
  $: if (candleSeries && dataStore.candles) {
    updateChartData();
  }
  
  // Public methods exposed via binding
  export function fitContent() {
    chart?.timeScale().fitContent();
  }
  
  export function showAllCandles() {
    if (!chart || dataStore.candles.length === 0) return;
    
    const firstCandle = dataStore.candles[0];
    const lastCandle = dataStore.candles[dataStore.candles.length - 1];
    
    // Add small buffer to ensure all candles are visible
    const buffer = (lastCandle.time as number - firstCandle.time as number) * 0.05;
    
    chart.timeScale().setVisibleRange({
      from: (firstCandle.time as number) - buffer,
      to: (lastCandle.time as number) + buffer
    });
    
    // Update visible range in dataStore
    dataStore.updateVisibleRange(
      (firstCandle.time as number) - buffer,
      (lastCandle.time as number) + buffer
    );
  }
  
  export function setVisibleRange(from: number, to: number) {
    if (chartStore.config.granularity === '1d') {
      console.log('[ChartCanvas] Setting visible range for 1d:', {
        from: new Date(from * 1000).toISOString(),
        to: new Date(to * 1000).toISOString(),
        rangeInDays: (to - from) / 86400
      });
    }
    chart?.timeScale().setVisibleRange({ from, to });
  }
  
  export function getChart(): IChartApi | null {
    return chart;
  }
  
  export function getSeries(): ISeriesApi<'Candlestick'> | null {
    return candleSeries;
  }
</script>

<div 
  bind:this={container} 
  class="chart-canvas"
  style:width={width ? `${width}px` : '100%'}
  style:height={height ? `${height}px` : '100%'}
/>

<style>
  .chart-canvas {
    position: relative;
    overflow: hidden;
  }
</style>