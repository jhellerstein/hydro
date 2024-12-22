use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::hydroflow_crate::tracing_options::TracingOptions;
use hydro_deploy::{Deployment, Host};
use hydro_lang::deploy::TrybuildHost;
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

// run with no args for localhost, with `gcp <GCP PROJECT>` for GCP
#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let (create_host, rustflags): (HostCreator, &'static str) = if host_arg == *"gcp" {
        let project = std::env::args().nth(2).unwrap();
        let network = Arc::new(RwLock::new(GcpNetwork::new(&project, None)));

        (
            Box::new(move |deployment| -> Arc<dyn Host> {
                let startup_script = "sudo sh -c 'apt update && apt install -y linux-perf binutils && echo -1 > /proc/sys/kernel/perf_event_paranoid && echo 0 > /proc/sys/kernel/kptr_restrict'";
                deployment
                    .GcpComputeEngineHost()
                    .project(&project)
                    .machine_type("e2-micro")
                    .image("debian-cloud/debian-11")
                    .region("us-west1-a")
                    .network(network.clone())
                    .startup_script(startup_script)
                    .add()
            }),
            "-C opt-level=3 -C codegen-units=1 -C strip=none -C debuginfo=2 -C lto=off",
        )
    } else {
        let localhost = deployment.Localhost();
        (
            Box::new(move |_| -> Arc<dyn Host> { localhost.clone() }),
            "-C opt-level=3 -C codegen-units=1 -C strip=none -C debuginfo=2 -C lto=off",
        )
    };

    let builder = hydro_lang::FlowBuilder::new();
    let (cluster, leader) = hydro_test::cluster::compute_pi::compute_pi(&builder, 8192);

    // Uncomment below, change .bin("counter_compute_pi") in order to track cardinality per operation
    // dbg!(builder.with_default_optimize()
    //     .optimize_with(|ir| profiling(ir, RuntimeData::new("FAKE"), RuntimeData::new("FAKE")))
    //     .ir());

    let _nodes = builder
        .with_process(
            &leader,
            TrybuildHost::new(create_host(&mut deployment))
                .rustflags(rustflags)
                .tracing(
                    TracingOptions::builder()
                        .perf_raw_outfile("leader.perf.data")
                        .dtrace_outfile("leader.stacks")
                        .fold_outfile("leader.data.folded")
                        .flamegraph_outfile("leader.svg")
                        .frequency(128)
                        .build(),
                ),
        )
        .with_cluster(
            &cluster,
            (0..8).map(|idx| {
                TrybuildHost::new(create_host(&mut deployment))
                    .rustflags(rustflags)
                    .tracing(
                        TracingOptions::builder()
                            .perf_raw_outfile(format!("cluster{}.perf.data", idx))
                            .dtrace_outfile(format!("cluster{}.leader.stacks", idx))
                            .fold_outfile(format!("cluster{}.data.folded", idx))
                            .flamegraph_outfile(format!("cluster{}.svg", idx))
                            .frequency(128)
                            .build(),
                    )
            }),
        )
        .deploy(&mut deployment);
    deployment.run_ctrl_c().await.unwrap();
}