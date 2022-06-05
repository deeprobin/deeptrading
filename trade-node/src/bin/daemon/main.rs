use std::{panic, time::Duration};

use atty::Stream;
use colored::Colorize;
use dotenv::dotenv;
use opentelemetry::global;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_core::LevelFilter;
use tracing_subscriber::{self, prelude::__tracing_subscriber_SubscriberExt, Layer, Registry};
use trade_node::{
    config::{NodeConfig, NodeEnvironment, TracingMode},
    Node,
};

const ASCII_ART: &str = r#"
██████╗ ████████╗    ███╗   ██╗ ██████╗ ██████╗ ███████╗
██╔══██╗╚══██╔══╝    ████╗  ██║██╔═══██╗██╔══██╗██╔════╝
██║  ██║   ██║       ██╔██╗ ██║██║   ██║██║  ██║█████╗  
██║  ██║   ██║       ██║╚██╗██║██║   ██║██║  ██║██╔══╝  
██████╔╝   ██║       ██║ ╚████║╚██████╔╝██████╔╝███████╗
╚═════╝    ╚═╝       ╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚══════╝                                           
"#;

fn main() {
    print_tty_header();

    dotenv().ok();
    let config = envy::prefixed("DTN_")
        .from_env::<NodeConfig>()
        .expect("Failed to parse environment variables");

    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    // Create a new OpenTelemetry pipeline
    let mut pipeline_builder = opentelemetry_jaeger::new_pipeline()
        .with_service_name("trade-node")
        .with_auto_split_batch(true);

    if let Some(ref jaeger_agent_endpoint) = config.jaeger_agent_endpoint {
        pipeline_builder = pipeline_builder.with_agent_endpoint(jaeger_agent_endpoint);
    }
    if let Some(ref jaeger_collector_endpoint) = config.jaeger_collector_endpoint {
        pipeline_builder = pipeline_builder.with_collector_endpoint(jaeger_collector_endpoint);
    }

    let tracing_mode = config.tracing_mode.unwrap_or_else(|| {
        if config.environment == NodeEnvironment::Development {
            TracingMode::Simple
        } else {
            TracingMode::Batch
        }
    });
    let tracer = match tracing_mode {
        TracingMode::Simple => pipeline_builder.install_simple(),
        TracingMode::Batch => pipeline_builder.install_simple(),
    }
    .expect("Failed to initialize OpenTelemetry-Jaeger pipeline");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::TRACE));

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    info!("Environment: {:?}", config.environment);
    if cfg!(debug_assertions) && config.environment != NodeEnvironment::Development {
        warn!("Running in debug mode, but environment is not development");
    }

    init_runtime(config);
    global::shutdown_tracer_provider();
    panic::set_hook(Box::new(|_| {
        global::shutdown_tracer_provider();
    }));
}

fn init_runtime(config: NodeConfig) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("node-thread")
        .build()
        .expect("Cannot create runtime");

    rt.block_on(run(config));
}

async fn run(config: NodeConfig) {
    let node_arc = Node::new(config);

    let (shutdown_send, shutdown_recv) = mpsc::unbounded_channel();

    let node_handle = tokio::spawn(async move {
        node_arc.run(shutdown_recv).await;
    });

    let shutdown_handle = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            shutdown_send
                .send(())
                .expect("Cannot send shutdown signal to node thread");

            info!("Received CTRL-C, shutting down");
            tokio::time::sleep(Duration::from_secs(10)).await;
            warn!("Shutdown takes longer as expected");
            tokio::time::sleep(Duration::from_secs(10)).await;

            error!("Shutdown timeout reached, forcing shutdown");
            std::process::exit(-1);
        } else {
            warn!("Cannot install CTRL+C handler");
        }
    });

    tokio::select! {
        _ = shutdown_handle => {},
        _ = node_handle => {},
    }
}

fn print_tty_header() {
    if atty::is(Stream::Stdout) {
        println!("{}", ASCII_ART.cyan().bold());
        println!();
        println!(
            "{}",
            "--------------------------------------------------------------------------------"
                .bright_red()
        );
        println!();
        println!(
            "{}",
            "⚠ This service decides on purchases and sales of trade goods.".bright_red()
        );
        println!(
            "{}",
            "⚠ This service is part of a critical infrastructure.".bright_red()
        );
        println!();
        println!(
            "{}",
            "--------------------------------------------------------------------------------"
                .bright_red()
        );
        println!();
    }
}
