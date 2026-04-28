use cloudvibe_database_operator::api::v1alpha1::{DatabaseAccess, DatabaseInstance};
use cloudvibe_database_operator::{controller, http, telemetry};
use kube::{Client, CustomResourceExt};
use std::{net::SocketAddr, process::ExitCode};
use tracing::{error, info};

const SERVICE_NAME: &str = "cloudvibe-database-operator";

#[tokio::main]
async fn main() -> ExitCode {
    let _telemetry = match telemetry::init(SERVICE_NAME) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("failed to initialize telemetry: {err}");
            return ExitCode::FAILURE;
        }
    };

    if std::env::args().nth(1).as_deref() == Some("export-crds") {
        return export_crds();
    }

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "operator exited");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    let state = http::HttpState::new();
    let server = tokio::spawn(http::serve(addr, state));

    let client = Client::try_default().await?;
    info!("starting controller");
    tokio::select! {
        _ = controller::run(client) => Ok(()),
        result = server => {
            result??;
            Ok(())
        },
        signal = tokio::signal::ctrl_c() => {
            signal?;
            info!("shutdown signal received");
            Ok(())
        }
    }
}

fn export_crds() -> ExitCode {
    let documents = [DatabaseInstance::crd(), DatabaseAccess::crd()];
    for (idx, crd) in documents.iter().enumerate() {
        if idx > 0 {
            println!("---");
        }
        match serde_yml::to_string(crd) {
            Ok(yaml) => print!("{yaml}"),
            Err(err) => {
                eprintln!("failed to serialize crd: {err}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
