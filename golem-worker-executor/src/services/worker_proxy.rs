// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::grpc::authorised_grpc_request;
use async_trait::async_trait;
use bincode::{Decode, Encode};
use golem_api_grpc::proto::golem::worker::v1::worker_service_client::WorkerServiceClient;
use golem_api_grpc::proto::golem::worker::v1::{
    fork_worker_response, invoke_and_await_typed_response, invoke_response, resume_worker_response,
    revert_worker_response, update_worker_response, worker_error, ForkWorkerRequest,
    InvokeAndAwaitRequest, InvokeAndAwaitTypedResponse, InvokeRequest, InvokeResponse,
    ResumeWorkerRequest, ResumeWorkerResponse, RevertWorkerRequest, RevertWorkerResponse,
    UpdateWorkerRequest, UpdateWorkerResponse, WorkerError,
};
use golem_api_grpc::proto::golem::worker::{InvokeParameters, UpdateMode};
use golem_common::client::{GrpcClient, GrpcClientConfig};
use golem_common::model::invocation_context::InvocationContextStack;
use golem_common::model::oplog::OplogIndex;
use golem_common::model::{ComponentVersion, IdempotencyKey, OwnedWorkerId, RetryConfig, WorkerId};
use golem_service_base::error::worker_executor::WorkerExecutorError;
use golem_service_base::model::RevertWorkerTarget;
use golem_wasm_rpc::{Value, ValueAndType, WitValue};
use http::Uri;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;
use tracing::debug;
use uuid::Uuid;

#[async_trait]
pub trait WorkerProxy: Send + Sync {
    async fn invoke_and_await(
        &self,
        owned_worker_id: &OwnedWorkerId,
        idempotency_key: Option<IdempotencyKey>,
        function_name: String,
        function_params: Vec<WitValue>,
        caller_worker_id: WorkerId,
        caller_args: Vec<String>,
        caller_env: HashMap<String, String>,
        caller_wasi_config_vars: BTreeMap<String, String>,
        caller_stack: InvocationContextStack,
    ) -> Result<Option<ValueAndType>, WorkerProxyError>;

    async fn invoke(
        &self,
        owned_worker_id: &OwnedWorkerId,
        idempotency_key: Option<IdempotencyKey>,
        function_name: String,
        function_params: Vec<WitValue>,
        caller_worker_id: WorkerId,
        caller_args: Vec<String>,
        caller_env: HashMap<String, String>,
        caller_wasi_config_vars: BTreeMap<String, String>,
        caller_stack: InvocationContextStack,
    ) -> Result<(), WorkerProxyError>;

    async fn update(
        &self,
        owned_worker_id: &OwnedWorkerId,
        target_version: ComponentVersion,
        mode: UpdateMode,
    ) -> Result<(), WorkerProxyError>;

    async fn resume(&self, owned_worker_id: &WorkerId, force: bool)
        -> Result<(), WorkerProxyError>;

    async fn fork_worker(
        &self,
        source_worker_id: &WorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cutoff: &OplogIndex,
    ) -> Result<(), WorkerProxyError>;

    async fn revert(
        &self,
        worker_id: &WorkerId,
        target: RevertWorkerTarget,
    ) -> Result<(), WorkerProxyError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum WorkerProxyError {
    BadRequest(Vec<String>),
    Unauthorized(String),
    LimitExceeded(String),
    NotFound(String),
    AlreadyExists(String),
    InternalError(WorkerExecutorError),
}

impl From<WorkerProxyError> for WorkerExecutorError {
    fn from(value: WorkerProxyError) -> Self {
        match value {
            WorkerProxyError::BadRequest(errors) => {
                WorkerExecutorError::invalid_request(errors.join(", "))
            }
            WorkerProxyError::Unauthorized(error) => WorkerExecutorError::unknown(error),
            WorkerProxyError::LimitExceeded(error) => WorkerExecutorError::unknown(error),
            WorkerProxyError::NotFound(error) => WorkerExecutorError::unknown(error),
            WorkerProxyError::AlreadyExists(error) => WorkerExecutorError::unknown(error),
            WorkerProxyError::InternalError(error) => error,
        }
    }
}

impl Error for WorkerProxyError {}

impl Display for WorkerProxyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerProxyError::BadRequest(errors) => write!(f, "Bad request: {}", errors.join(", ")),
            WorkerProxyError::Unauthorized(error) => write!(f, "Unauthorized: {error}"),
            WorkerProxyError::LimitExceeded(error) => write!(f, "Limit exceeded: {error}"),
            WorkerProxyError::NotFound(error) => write!(f, "Not found: {error}"),
            WorkerProxyError::AlreadyExists(error) => write!(f, "Already exists: {error}"),
            WorkerProxyError::InternalError(error) => write!(f, "Internal error: {error}"),
        }
    }
}

impl From<tonic::transport::Error> for WorkerProxyError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::InternalError(WorkerExecutorError::unknown(format!(
            "gRPC Transport error: {value}"
        )))
    }
}

impl From<tonic::Status> for WorkerProxyError {
    fn from(value: tonic::Status) -> Self {
        Self::InternalError(WorkerExecutorError::unknown(format!("gRPC error: {value}")))
    }
}

impl From<WorkerError> for WorkerProxyError {
    fn from(value: WorkerError) -> Self {
        match value.error {
            Some(worker_error::Error::BadRequest(body)) => {
                WorkerProxyError::BadRequest(body.errors)
            }
            Some(worker_error::Error::Unauthorized(body)) => {
                WorkerProxyError::Unauthorized(body.error)
            }
            Some(worker_error::Error::LimitExceeded(body)) => {
                WorkerProxyError::LimitExceeded(body.error)
            }
            Some(worker_error::Error::NotFound(body)) => WorkerProxyError::NotFound(body.error),
            Some(worker_error::Error::AlreadyExists(body)) => {
                WorkerProxyError::AlreadyExists(body.error)
            }
            Some(worker_error::Error::InternalError(worker_executor_error)) => {
                WorkerProxyError::InternalError(worker_executor_error.try_into().unwrap_or(
                    WorkerExecutorError::unknown(
                        "Unknown error from the worker executor".to_string(),
                    ),
                ))
            }
            None => WorkerProxyError::InternalError(WorkerExecutorError::unknown(
                "Empty error response from the worker API".to_string(),
            )),
        }
    }
}

impl From<WorkerExecutorError> for WorkerProxyError {
    fn from(value: WorkerExecutorError) -> Self {
        WorkerProxyError::InternalError(value)
    }
}

pub struct RemoteWorkerProxy {
    client: GrpcClient<WorkerServiceClient<Channel>>,
    access_token: Uuid,
}

impl RemoteWorkerProxy {
    pub fn new(
        endpoint: Uri,
        access_token: Uuid,
        retry_config: RetryConfig,
        connect_timeout: Duration,
    ) -> Self {
        Self {
            client: GrpcClient::new(
                "worker_service",
                |channel| {
                    WorkerServiceClient::new(channel)
                        .send_compressed(CompressionEncoding::Gzip)
                        .accept_compressed(CompressionEncoding::Gzip)
                },
                endpoint,
                GrpcClientConfig {
                    retries_on_unavailable: retry_config,
                    connect_timeout,
                },
            ),
            access_token,
        }
    }
}

#[async_trait]
impl WorkerProxy for RemoteWorkerProxy {
    async fn invoke_and_await(
        &self,
        owned_worker_id: &OwnedWorkerId,
        idempotency_key: Option<IdempotencyKey>,
        function_name: String,
        function_params: Vec<WitValue>,
        caller_worker_id: WorkerId,
        caller_args: Vec<String>,
        caller_env: HashMap<String, String>,
        caller_wasi_config_vars: BTreeMap<String, String>,
        caller_stack: InvocationContextStack,
    ) -> Result<Option<ValueAndType>, WorkerProxyError> {
        debug!(
            "Invoking remote worker function {function_name} with parameters {function_params:?}"
        );

        let proto_params = function_params
            .into_iter()
            .map(|param| {
                let value: Value = param.into();
                value.into()
            })
            .collect();
        let invoke_parameters = Some(InvokeParameters {
            params: proto_params,
        });

        let response: InvokeAndAwaitTypedResponse = self
            .client
            .call("invoke_and_await_typed", move |client| {
                Box::pin(client.invoke_and_await_typed(authorised_grpc_request(
                    InvokeAndAwaitRequest {
                        worker_id: Some(owned_worker_id.worker_id().into_target_worker_id().into()),
                        idempotency_key: idempotency_key.clone().map(|k| k.into()),
                        function: function_name.clone(),
                        invoke_parameters: invoke_parameters.clone(),
                        context: Some(golem_api_grpc::proto::golem::worker::InvocationContext {
                            parent: Some(caller_worker_id.clone().into()),
                            args: caller_args.clone(),
                            env: caller_env.clone(),
                            wasi_config_vars: Some(caller_wasi_config_vars.clone().into()),
                            tracing: Some(caller_stack.clone().into()),
                        }),
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(invoke_and_await_typed_response::Result::Success(result)) => {
                let result = result
                    .result
                    .map(|proto_vnt| {
                        ValueAndType::try_from(proto_vnt).map_err(|e| {
                            WorkerProxyError::InternalError(WorkerExecutorError::unknown(format!(
                                "Failed to parse invocation result value: {e}"
                            )))
                        })
                    })
                    .transpose()?;
                Ok(result)
            }
            Some(invoke_and_await_typed_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown("Empty response through the worker API".to_string()),
            )),
        }
    }

    async fn invoke(
        &self,
        owned_worker_id: &OwnedWorkerId,
        idempotency_key: Option<IdempotencyKey>,
        function_name: String,
        function_params: Vec<WitValue>,
        caller_worker_id: WorkerId,
        caller_args: Vec<String>,
        caller_env: HashMap<String, String>,
        caller_wasi_config_vars: BTreeMap<String, String>,
        caller_stack: InvocationContextStack,
    ) -> Result<(), WorkerProxyError> {
        debug!("Invoking remote worker function {function_name} with parameters {function_params:?} without awaiting for the result");

        let proto_params = function_params
            .into_iter()
            .map(|param| {
                let value: Value = param.into();
                value.into()
            })
            .collect();
        let invoke_parameters = Some(InvokeParameters {
            params: proto_params,
        });

        let response: InvokeResponse = self
            .client
            .call("invoke", move |client| {
                Box::pin(client.invoke(authorised_grpc_request(
                    InvokeRequest {
                        worker_id: Some(owned_worker_id.worker_id().into_target_worker_id().into()),
                        idempotency_key: idempotency_key.clone().map(|k| k.into()),
                        function: function_name.clone(),
                        invoke_parameters: invoke_parameters.clone(),
                        context: Some(golem_api_grpc::proto::golem::worker::InvocationContext {
                            parent: Some(caller_worker_id.clone().into()),
                            args: caller_args.clone(),
                            env: caller_env.clone(),
                            wasi_config_vars: Some(caller_wasi_config_vars.clone().into()),
                            tracing: Some(caller_stack.clone().into()),
                        }),
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(invoke_response::Result::Success(_)) => Ok(()),
            Some(invoke_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown("Empty response through the worker API".to_string()),
            )),
        }
    }

    async fn update(
        &self,
        owned_worker_id: &OwnedWorkerId,
        target_version: ComponentVersion,
        mode: UpdateMode,
    ) -> Result<(), WorkerProxyError> {
        debug!("Updating remote worker to version {target_version} in {mode:?} mode");

        let response: UpdateWorkerResponse = self
            .client
            .call("update_worker", move |client| {
                Box::pin(client.update_worker(authorised_grpc_request(
                    UpdateWorkerRequest {
                        worker_id: Some(owned_worker_id.worker_id().into()),
                        target_version,
                        mode: mode as i32,
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(update_worker_response::Result::Success(_)) => Ok(()),
            Some(update_worker_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown("Empty response through the worker API".to_string()),
            )),
        }
    }

    async fn resume(&self, worker_id: &WorkerId, force: bool) -> Result<(), WorkerProxyError> {
        debug!("Resuming remote worker");

        let response: ResumeWorkerResponse = self
            .client
            .call("resume_worker", move |client| {
                Box::pin(client.resume_worker(authorised_grpc_request(
                    ResumeWorkerRequest {
                        worker_id: Some(worker_id.clone().into()),
                        force: Some(force),
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(resume_worker_response::Result::Success(_)) => Ok(()),
            Some(resume_worker_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown("Empty response through the worker API".to_string()),
            )),
        }
    }

    async fn fork_worker(
        &self,
        source_worker_id: &WorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cutoff: &OplogIndex,
    ) -> Result<(), WorkerProxyError> {
        debug!("Forking remote worker");

        let response = self
            .client
            .call("fork_worker", move |client| {
                Box::pin(client.fork_worker(authorised_grpc_request(
                    ForkWorkerRequest {
                        source_worker_id: Some(source_worker_id.clone().into()),
                        target_worker_id: Some(target_worker_id.clone().into()),
                        oplog_index_cutoff: u64::from(*oplog_index_cutoff),
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(fork_worker_response::Result::Success(_)) => Ok(()),
            Some(fork_worker_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown(
                    "Empty response through the worker API during fork".to_string(),
                ),
            )),
        }
    }

    async fn revert(
        &self,
        worker_id: &WorkerId,
        target: RevertWorkerTarget,
    ) -> Result<(), WorkerProxyError> {
        let response: RevertWorkerResponse = self
            .client
            .call("revert_worker", move |client| {
                Box::pin(client.revert_worker(authorised_grpc_request(
                    RevertWorkerRequest {
                        worker_id: Some(worker_id.clone().into()),
                        target: Some(target.clone().into()),
                    },
                    &self.access_token,
                )))
            })
            .await?
            .into_inner();

        match response.result {
            Some(revert_worker_response::Result::Success(_)) => Ok(()),
            Some(revert_worker_response::Result::Error(error)) => Err(error.into()),
            None => Err(WorkerProxyError::InternalError(
                WorkerExecutorError::unknown("Empty response through the worker API".to_string()),
            )),
        }
    }
}
