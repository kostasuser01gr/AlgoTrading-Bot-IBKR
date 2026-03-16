from __future__ import annotations

from dataclasses import dataclass

from adaptive_quant.features import rolling_features


@dataclass(slots=True)
class BacktestReport:
    total_return: float
    max_drawdown: float
    trade_count: int
    win_rate: float


def run_backtest(prices: list[float], window: int = 5, threshold: float = 0.02) -> BacktestReport:
    if len(prices) < window + 1:
        return BacktestReport(total_return=0.0, max_drawdown=0.0, trade_count=0, win_rate=0.0)

    features = rolling_features(prices, window=window)
    trade_returns: list[float] = []

    for index, snapshot in enumerate(features[:-1], start=window - 1):
        if snapshot.momentum > threshold:
            next_price = prices[index + 1]
            current_price = prices[index]
            trade_returns.append((next_price - current_price) / current_price)

    equity_curve = 0.0
    peak = 0.0
    max_drawdown = 0.0
    for trade_return in trade_returns:
        equity_curve += trade_return
        peak = max(peak, equity_curve)
        max_drawdown = min(max_drawdown, equity_curve - peak)

    trade_count = len(trade_returns)
    wins = len([trade_return for trade_return in trade_returns if trade_return > 0.0])
    win_rate = wins / trade_count if trade_count else 0.0

    return BacktestReport(
        total_return=sum(trade_returns),
        max_drawdown=max_drawdown,
        trade_count=trade_count,
        win_rate=win_rate,
    )
