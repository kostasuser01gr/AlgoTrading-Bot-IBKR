from __future__ import annotations

import json
from argparse import ArgumentParser
from dataclasses import asdict

from adaptive_quant.backtest import run_backtest


def main() -> None:
    parser = ArgumentParser(description="Run a lightweight quant backtest")
    parser.add_argument(
        "--prices",
        default="[100,102,104,103,105,108,110,109,111,115]",
        help="JSON array of prices",
    )
    args = parser.parse_args()

    prices = json.loads(args.prices)
    report = run_backtest(prices)
    print(json.dumps(asdict(report), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
