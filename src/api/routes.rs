use axum::{
	routing::{get, post},
	Router,
	Json,
};
use crate::model::{
	optimizer::{OptimizationModel, OptimizationConfig},
	encoder::AssemblyEncoder,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OptimizeRequest {
	assembly: String,
}

#[derive(Serialize)]
pub struct OptimizeResponse {
	optimized_assembly: String,
}

async fn optimize_assembly(
	Json(request): Json<OptimizeRequest>,
) -> Json<OptimizeResponse> {
	let config = OptimizationConfig::default();
	let model = OptimizationModel::new(config);
	let mut encoder = AssemblyEncoder::new();

	let input_tokens = encoder.encode(&request.assembly);
	let _optimized_tokens = model.optimize(&input_tokens);

	// For now, return the input as we haven't implemented the full optimization
	Json(OptimizeResponse {
		optimized_assembly: request.assembly,
	})
}

async fn health_check() -> &'static str {
	"OK"
}

pub fn setup_router() -> Router {
	Router::new()
		.route("/optimize", post(optimize_assembly))
		.route("/health", get(health_check))
}