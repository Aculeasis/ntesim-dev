"""Prepare maintained current inputs for the native/WASM transport boundary."""
import json
from pathlib import Path
from verification_workspace import write_review_receipt

ROOT = Path(__file__).resolve().parents[1]


def prepare_batch_parity_inputs():
    """Produce current transport fixtures, without certifying domain arithmetic.

    Match the existing research batch workflow's seed/budget expansion; the
    parity consumers require completed native/WASM runs themselves. Historical
    verification receipts are never used as input or refreshed here.
    """
    scenario = json.loads((ROOT / 'examples/research-crit.json').read_bytes())
    batch = json.loads((ROOT / 'examples/research-game-batch.json').read_bytes())
    batch.update(seeds=list(range(1000)), max_total_events=300000,
                 max_run_receipt_bytes=8 * 1024 * 1024)
    for name, value in (('scenario-request.json', scenario), ('pilot-request.json', batch)):
        write_review_receipt(ROOT / 'reviews/q10-research-batch' / name, value, refresh=True)


if __name__ == '__main__':
    import argparse
    from cpu_limit import limit_cpus
    parser = argparse.ArgumentParser(description='Prepare current cache-only native/WASM parity inputs')
    parser.add_argument('--prepare-batch-inputs', action='store_true', required=True)
    parser.parse_args()
    limit_cpus()
    prepare_batch_parity_inputs()
