use std::default::Default;
use std::sync::{Arc, Mutex};

use clap::{arg, Parser};
use tonic::{transport::Server, Request, Response, Status};

use curaengine_grpc_definitions::slots::gcode_paths::v0::modify::g_code_paths_modify_service_server::{
    GCodePathsModifyService, GCodePathsModifyServiceServer,
};
use curaengine_grpc_definitions::slots::gcode_paths::v0::modify::CallRequest as GCodePathsModifyRequest;
use curaengine_grpc_definitions::slots::gcode_paths::v0::modify::CallResponse as GCodePathsModifyResponse;
use curaengine_grpc_definitions::slots::broadcast::v0::broadcast_service_server::{
    BroadcastService, BroadcastServiceServer,
};
use curaengine_grpc_definitions::slots::broadcast::v0::BroadcastServiceSettingsRequest;
use curaengine_grpc_definitions::slots::handshake::v0::handshake_service_server::{
    HandshakeService, HandshakeServiceServer,
};
use curaengine_grpc_definitions::slots::handshake::v0::CallRequest as HandshakeRequest;
use curaengine_grpc_definitions::slots::handshake::v0::CallResponse as HandshakeResponse;

use curaengine_grpc_definitions::v0::SlotId;

#[derive(Default)]
struct HandshakeServicer {}

#[tonic::async_trait]
impl HandshakeService for HandshakeServicer {
    async fn call(
        &self,
        request: Request<HandshakeRequest>,
    ) -> Result<Response<HandshakeResponse>, Status> {
        println!("Received a handshake: {:?}", request);

        let response = HandshakeResponse {
            plugin_version: env!("CARGO_PKG_VERSION").to_string(),
            plugin_name: env!("CARGO_PKG_NAME").to_string(),
            slot_version_range: ">=0.1.0 <0.2.0".to_string(),
            broadcast_subscriptions: vec![
                SlotId::SettingsBroadcast as i32,
                SlotId::GcodePathsModify as i32,
            ],
        };
        Result::Ok(Response::new(response))
    }
}

struct BroadcastServicer {
    state: Arc<Mutex<State>>,
}

#[tonic::async_trait]
impl BroadcastService for BroadcastServicer {
    async fn broadcast_settings(
        &self,
        request: Request<BroadcastServiceSettingsRequest>,
    ) -> Result<tonic::Response<()>, Status> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| Status::aborted("unable to access shared state"))?;

        let plugin_prefix = format!(
            "_plugin__{plugin_name}__{major}_{minor}_{patch}__",
            plugin_name = env!("CARGO_PKG_NAME").to_string(),
            major = env!("CARGO_PKG_VERSION_MAJOR").to_string(),
            minor = env!("CARGO_PKG_VERSION_MINOR").to_string(),
            patch = env!("CARGO_PKG_VERSION_PATCH").to_string(),
        );

        state.extruder_max_flow = request
            .get_ref()
            .extruder_settings
            .iter()
            .map(|settings| -> Result<f64, Status> {
                String::from_utf8(
                    settings
                        .settings
                        .get(&format!("{plugin_prefix}max_flow"))
                        .ok_or(Status::aborted("unable to parse max flow"))?
                        .to_vec(),
                )
                .map_err(|_| Status::aborted("unable to parse max flow"))?
                .parse::<f64>()
                .map_err(|_| Status::aborted("unable to parse max flow"))
            })
            .collect::<Result<Vec<f64>, Status>>()?;

        Result::Ok(Response::new(()))
    }
}

struct GCodePathsServicer {
    state: Arc<Mutex<State>>,
}

#[tonic::async_trait]
impl GCodePathsModifyService for GCodePathsServicer {
    async fn call(
        &self,
        request: Request<GCodePathsModifyRequest>,
    ) -> Result<Response<GCodePathsModifyResponse>, Status> {
        request.remote_addr();

        let extruder = request.get_ref().extruder_nr;
        let max_flow = self
            .state
            .lock()
            .map_err(|_| Status::aborted("unable to access shared state"))?
            .extruder_max_flow
            .get(extruder as usize)
            .ok_or(Status::aborted("unable to get max flow"))?
            * 1e6;

        // Do not modify the gcode paths if the max flow is not set.
        if max_flow == 0.0 {
            return Result::Ok(Response::new(GCodePathsModifyResponse {
                gcode_paths: request.into_inner().gcode_paths,
            }));
        }

        let gcode_paths = request
            .into_inner()
            .gcode_paths
            .into_iter()
            .map(move |mut gcode_path| {
                let extrusion_volume_per_mm = gcode_path.flow
                    * gcode_path.line_width as f64
                    * gcode_path.layer_thickness as f64
                    * gcode_path.flow_ratio;
                let speed = gcode_path.speed_factor
                    * gcode_path
                        .speed_derivatives
                        .as_ref()
                        .map_or(1.0, |speed_derivatives| speed_derivatives.velocity);
                let flow = extrusion_volume_per_mm * speed;

                if flow > max_flow {
                    gcode_path
                        .speed_derivatives
                        .as_mut()
                        .map(|speed_derivatives| {
                            speed_derivatives.velocity =
                                max_flow / extrusion_volume_per_mm / gcode_path.speed_factor;
                        });
                }

                return gcode_path;
            })
            .collect::<Vec<_>>();

        Result::Ok(Response::new(GCodePathsModifyResponse { gcode_paths }))
    }
}

#[derive(Default)]
struct State {
    extruder_max_flow: Vec<f64>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: String,
    #[arg(short, long, default_value_t = String::from("[::]"))]
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let address: std::net::SocketAddr =
        format!("{address}:{port}", address = args.address, port = args.port,).parse()?;

    println!("Listening on {}", address);

    let state = Arc::new(Mutex::from(State::default()));

    Server::builder()
        .add_service(HandshakeServiceServer::new(HandshakeServicer {}))
        .add_service(BroadcastServiceServer::new(BroadcastServicer {
            state: state.clone(),
        }))
        .add_service(GCodePathsModifyServiceServer::new(GCodePathsServicer {
            state: state.clone(),
        }))
        .serve(address)
        .await?;
    Ok(())
}
