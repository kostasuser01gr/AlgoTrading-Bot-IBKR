from adaptive_quant.backtest import run_backtest
from adaptive_quant.features import rolling_features


def test_rolling_features_returns_snapshots() -> None:
    snapshots = rolling_features([100, 102, 104, 106, 108, 110], window=3)
    assert snapshots
    assert snapshots[-1].momentum > 0


def test_backtest_returns_non_negative_win_rate() -> None:
    report = run_backtest([100, 102, 104, 103, 105, 107, 109, 111], window=3, threshold=0.01)
    assert report.trade_count >= 1
    assert 0.0 <= report.win_rate <= 1.0

