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

use async_trait::async_trait;
use golem_api_grpc::proto::golem::component::v1::component_service_client::ComponentServiceClient;
use golem_api_grpc::proto::golem::component::v1::{
    get_component_metadata_response, GetLatestComponentRequest,
};
use golem_common::cache::{BackgroundEvictionMode, Cache, FullCacheEvictionMode, SimpleCache};
use golem_common::client::{GrpcClient, GrpcClientConfig};
use golem_common::model::auth::ProjectAction;
use golem_common::model::auth::{AuthCtx, Namespace};
use golem_common::model::{AccountId, ComponentId, ProjectId};
use golem_common::retries::with_retries;
use golem_service_base::clients::auth::{AuthService as BaseAuthService, AuthServiceError};
use golem_worker_executor::services::golem_config::ComponentServiceGrpcConfig;
use std::time::Duration;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;
use tonic::Status;
use tracing::error;

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn get_account(&self, ctx: &AuthCtx) -> Result<AccountId, AuthServiceError>;

    async fn authorize_project_action(
        &self,
        project_id: &ProjectId,
        permission: ProjectAction,
        ctx: &AuthCtx,
    ) -> Result<Namespace, AuthServiceError>;

    async fn is_authorized_by_component(
        &self,
        component_id: &ComponentId,
        permission: ProjectAction,
        ctx: &AuthCtx,
    ) -> Result<Namespace, AuthServiceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DebuggingServiceAuthError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad Request: {}", .0.join(", "))]
    BadRequest(Vec<String>),
    #[error("Internal component service error: {0}")]
    Internal(String),
    #[error("Internal error: {0}")]
    FailedGrpcStatus(Status),
    #[error("Internal error: {0}")]
    FailedTransport(tonic::transport::Error),
}

impl From<Status> for DebuggingServiceAuthError {
    fn from(status: Status) -> Self {
        DebuggingServiceAuthError::FailedGrpcStatus(status)
    }
}

impl From<tonic::transport::Error> for DebuggingServiceAuthError {
    fn from(error: tonic::transport::Error) -> Self {
        DebuggingServiceAuthError::FailedTransport(error)
    }
}

impl From<golem_api_grpc::proto::golem::component::v1::ComponentError>
    for DebuggingServiceAuthError
{
    fn from(error: golem_api_grpc::proto::golem::component::v1::ComponentError) -> Self {
        use golem_api_grpc::proto::golem::component::v1::component_error::Error;
        match error.error {
            Some(Error::BadRequest(errors)) => DebuggingServiceAuthError::BadRequest(errors.errors),
            Some(Error::Unauthorized(error)) => {
                DebuggingServiceAuthError::Unauthorized(error.error)
            }
            Some(Error::LimitExceeded(error)) => DebuggingServiceAuthError::Forbidden(error.error),
            Some(Error::NotFound(error)) => DebuggingServiceAuthError::NotFound(error.error),
            Some(Error::AlreadyExists(error)) => DebuggingServiceAuthError::Internal(error.error),
            Some(Error::InternalError(error)) => DebuggingServiceAuthError::Internal(error.error),
            None => DebuggingServiceAuthError::Internal("Unknown error".to_string()),
        }
    }
}

pub struct GrpcAuthService {
    common_auth: BaseAuthService,
    component_service_grpc_config: ComponentServiceGrpcConfig,
    component_service_client: GrpcClient<ComponentServiceClient<Channel>>,
    component_project_cache: Cache<ComponentId, (), ProjectId, String>,
}

impl GrpcAuthService {
    pub fn new(
        common_auth: BaseAuthService,
        component_service_grpc_config: ComponentServiceGrpcConfig,
    ) -> Self {
        let component_service_client = GrpcClient::new(
            "component_service",
            |channel| {
                ComponentServiceClient::new(channel)
                    .send_compressed(CompressionEncoding::Gzip)
                    .accept_compressed(CompressionEncoding::Gzip)
            },
            component_service_grpc_config.uri(),
            GrpcClientConfig {
                retries_on_unavailable: component_service_grpc_config.retries.clone(),
                ..Default::default() // TODO
            },
        );

        // TODO configuration
        let component_project_cache = Cache::new(
            Some(10000),
            FullCacheEvictionMode::LeastRecentlyUsed(1),
            BackgroundEvictionMode::OlderThan {
                ttl: Duration::from_secs(60 * 60),
                period: Duration::from_secs(60),
            },
            "component_project",
        );

        Self {
            common_auth,
            component_service_grpc_config,
            component_service_client,
            component_project_cache,
        }
    }

    async fn get_project(
        &self,
        component_id: &ComponentId,
        metadata: &AuthCtx,
    ) -> Result<ProjectId, AuthServiceError> {
        let id = component_id.clone();
        let metadata = metadata.clone();
        let retries = self.component_service_grpc_config.retries.clone();
        let client = self.component_service_client.clone();

        self.component_project_cache
            .get_or_insert_simple(component_id, || {
                Box::pin(async move {
                    let result = with_retries(
                        "component",
                        "get_project",
                        Some(format!("{id}")),
                        &retries.clone(),
                        &(client.clone(), id.clone(), metadata.clone()),
                        |(client, id, metadata)| {
                            Box::pin(async move {
                                let response = client
                                    .call("get_latest_component", move |client| {
                                        let request = GetLatestComponentRequest {
                                            component_id: Some(id.clone().into()),
                                        };
                                        let request = with_metadata(request, metadata.clone());

                                        Box::pin(client.get_latest_component_metadata(request))
                                    })
                                    .await?
                                    .into_inner();

                                match response.result {
                                    None => Err(DebuggingServiceAuthError::Unauthorized(
                                        "Empty response".to_string(),
                                    )),
                                    Some(get_component_metadata_response::Result::Success(
                                        response,
                                    )) => response
                                        .component
                                        .and_then(|c| c.project_id)
                                        .and_then(|id| id.try_into().ok())
                                        .ok_or_else(|| {
                                            DebuggingServiceAuthError::Unauthorized(
                                                "Empty project id".to_string(),
                                            )
                                        }),
                                    Some(get_component_metadata_response::Result::Error(error)) => {
                                        let err = error.into();
                                        Err(err)
                                    }
                                }
                            })
                        },
                        is_retriable,
                    )
                    .await;

                    result.map_err(|e| {
                        error!("Getting project of component: {} - error: {}", id, e);
                        "Get project error".to_string()
                    })
                })
            })
            .await
            .map_err(AuthServiceError::Unauthorized)
    }
}

#[async_trait]
impl AuthService for GrpcAuthService {
    async fn get_account(&self, ctx: &AuthCtx) -> Result<AccountId, AuthServiceError> {
        self.common_auth.get_account(ctx).await
    }

    async fn authorize_project_action(
        &self,
        project_id: &ProjectId,
        permission: ProjectAction,
        ctx: &AuthCtx,
    ) -> Result<Namespace, AuthServiceError> {
        self.common_auth
            .authorize_project_action(project_id, permission, ctx)
            .await
    }

    async fn is_authorized_by_component(
        &self,
        component_id: &ComponentId,
        permission: ProjectAction,
        ctx: &AuthCtx,
    ) -> Result<Namespace, AuthServiceError> {
        let project_id = self.get_project(component_id, ctx).await?;

        self.authorize_project_action(&project_id, permission, ctx)
            .await
    }
}

fn is_retriable(error: &DebuggingServiceAuthError) -> bool {
    matches!(
        error,
        DebuggingServiceAuthError::FailedTransport(_)
            | DebuggingServiceAuthError::FailedGrpcStatus(_)
    )
}

pub fn with_metadata<T, I, K, V>(request: T, metadata: I) -> tonic::Request<T>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut req = tonic::Request::new(request);
    let req_metadata = req.metadata_mut();

    for (key, value) in metadata {
        let key = tonic::metadata::MetadataKey::from_bytes(key.as_ref().as_bytes());
        let value = value.as_ref().parse();
        if let (Ok(key), Ok(value)) = (key, value) {
            req_metadata.insert(key, value);
        }
    }

    req
}
