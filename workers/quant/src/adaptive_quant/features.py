from __future__ import annotations

from collections import deque
from dataclasses import dataclass
from statistics import fmean, pstdev


@dataclass(slots=True)
class FeatureSnapshot:
    momentum: float
    volatility: float
    zscore: float


def rolling_features(prices: list[float], window: int = 5) -> list[FeatureSnapshot]:
    if window <= 1:
        raise ValueError("window must be greater than 1")
    if len(prices) < window:
        return []

    values: deque[float] = deque(maxlen=window)
    snapshots: list[FeatureSnapshot] = []

    for price in prices:
        values.append(price)
        if len(values) < window:
            continue

        mean = fmean(values)
        volatility = pstdev(values) if len(values) > 1 else 0.0
        last = values[-1]
        first = values[0]
        momentum = (last - first) / first if first else 0.0
        zscore = 0.0 if volatility == 0.0 else (last - mean) / volatility
        snapshots.append(
            FeatureSnapshot(momentum=momentum, volatility=volatility, zscore=zscore)
        )

    return snapshots

