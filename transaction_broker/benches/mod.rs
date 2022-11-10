use criterion::{
    async_executor::AsyncStdExecutor, criterion_group, criterion_main, Criterion, Throughput,
};
use pprof::criterion::{Output, PProfProfiler};
use transaction_broker::{process_csv_txs, process_csv_txs_sync};

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_deposit, bench_deposit_withdraw, bench_deposit_dispute_resolve, bench_deposit_many_acc, bench_deposit_withdraw_many_acc, bench_deposit_dispute_resolve_many_acc,
}
criterion_main!(benches);

// TODO: https://www.jibbow.com/posts/criterion-flamegraphs/ try it
fn bench_deposit(c: &mut Criterion) {
    let mut group = c.benchmark_group("deposits");

    group.throughput(Throughput::Elements(180));

    group.bench_function("deposits", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit.csv",
                "../test_data/outputs/deposit.csv",
            )
        });
    });
    group.bench_function("deposits sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit.csv",
                "../test_data/outputs/deposit.csv",
            )
        });
    });
}

fn bench_deposit_withdraw(c: &mut Criterion) {
    let mut group = c.benchmark_group("deposit, withdraw");
    group.throughput(Throughput::Elements(180));

    group.bench_function("deposit withdraw", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit_withdraw.csv",
                "../test_data/outputs/deposit_withdraw.csv",
            )
        });
    });

    group.bench_function("deposit withdraw sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit_withdraw.csv",
                "../test_data/outputs/deposit_withdraw.csv",
            )
        });
    });
}

fn bench_deposit_dispute_resolve(c: &mut Criterion) {
    let mut group = c.benchmark_group("deposit, dispute, resolve");
    group.throughput(Throughput::Elements(180));

    group.bench_function("deposit dispute withdraw", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit_dispute_resolve.csv",
                "../test_data/outputs/deposit_dispute_resolve_accounts.csv",
            )
        });
    });

    group.bench_function("deposit dispute withdraw sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit_dispute_resolve.csv",
                "../test_data/outputs/deposit_dispute_resolve_accounts.csv",
            )
        });
    });
}

fn bench_deposit_many_acc(c: &mut Criterion) {
    let mut group = c.benchmark_group("deposits - multiple accs");
    group.throughput(Throughput::Elements(18000));

    group.bench_function("deposits", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit_many_acc.csv",
                "../test_data/outputs/deposit_accounts_many_acc.csv",
            )
        });
    });
    group.bench_function("deposits sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit_many_acc.csv",
                "../test_data/outputs/deposit_many_acc.csv",
            )
        });
    });
}

fn bench_deposit_withdraw_many_acc(c: &mut Criterion) {
    // 100 acc
    // 180 tx per acc
    let mut group = c.benchmark_group("deposit, withdraw - multiple acc");
    group.throughput(Throughput::Elements(18000));

    group.bench_function("deposit withdraw", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit_withdraw_many_acc.csv",
                "../test_data/outputs/deposit_withdraw_many_acc.csv",
            )
        });
    });

    group.bench_function("deposit withdraw sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit_withdraw_many_acc.csv",
                "../test_data/outputs/deposit_withdraw_many_acc.csv",
            )
        });
    });
}

fn bench_deposit_dispute_resolve_many_acc(c: &mut Criterion) {
    // 180 tx
    // 100 acc
    let mut group = c.benchmark_group("deposit, dispute, resolve - many acc");
    group.throughput(Throughput::Elements(18000));

    group.bench_function("deposit dispute withdraw", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs(
                "../test_data/inputs/deposit_dispute_resolve_many_acc.csv",
                "../test_data/outputs/deposit_dispute_resolve_many_acc.csv",
            )
        });
    });

    group.bench_function("deposit dispute withdraw sync", |b| {
        b.to_async(AsyncStdExecutor).iter(|| {
            process_csv_txs_sync(
                "../test_data/inputs/deposit_dispute_resolve_many_acc.csv",
                "../test_data/outputs/deposit_dispute_resolve_many_acc.csv",
            )
        });
    });
}
