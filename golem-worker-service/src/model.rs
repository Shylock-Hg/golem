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

use crate::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use crate::gateway_api_deployment::ApiSite;
use derive_more::FromStr;
use golem_common::model::regions::OplogRegion;
use golem_common::model::worker::WasiConfigVars;
use golem_common::model::{AccountId, PluginInstallationId, ScanCursor, WorkerId};
use golem_common::model::{ComponentVersion, ProjectId, Timestamp, WorkerStatus};
use golem_service_base::model::{ResourceMetadata, UpdateRecord};
use poem_openapi::{NewType, Object};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct WorkerMetadata {
    pub worker_id: WorkerId,
    pub project_id: ProjectId,
    pub created_by: AccountId,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub wasi_config_vars: WasiConfigVars,
    pub status: WorkerStatus,
    pub component_version: ComponentVersion,
    pub retry_count: u64,
    pub pending_invocation_count: u64,
    pub updates: Vec<UpdateRecord>,
    pub created_at: Timestamp,
    pub last_error: Option<String>,
    pub component_size: u64,
    pub total_linear_memory_size: u64,
    pub owned_resources: HashMap<u64, ResourceMetadata>,
    pub active_plugins: HashSet<PluginInstallationId>,
    /// Oplog regions that are skipped during the worker's state recovery, but describe
    /// the history of the worker. For example if an atomic region gets restarted, its partially
    /// recorded oplog entries will be skipped on retry.
    pub skipped_regions: Vec<OplogRegion>,
    /// Oplog regions permanently deleted from the workers using the revert functionality.
    pub deleted_regions: Vec<OplogRegion>,
}

impl TryFrom<golem_api_grpc::proto::golem::worker::WorkerMetadata> for WorkerMetadata {
    type Error = String;

    fn try_from(
        value: golem_api_grpc::proto::golem::worker::WorkerMetadata,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            worker_id: value.worker_id.ok_or("Missing worker_id")?.try_into()?,
            project_id: value.project_id.ok_or("Missing project_id")?.try_into()?,
            created_by: value.created_by.ok_or("Missing account_id")?.into(),
            args: value.args,
            env: value.env,
            wasi_config_vars: value
                .wasi_config_vars
                .ok_or("Missing wasi_config_vars field")?
                .into(),
            status: value.status.try_into()?,
            component_version: value.component_version,
            retry_count: value.retry_count,
            pending_invocation_count: value.pending_invocation_count,
            updates: value
                .updates
                .into_iter()
                .map(|update| update.try_into())
                .collect::<Result<Vec<UpdateRecord>, String>>()?,
            created_at: value.created_at.ok_or("Missing created_at")?.into(),
            last_error: value.last_error,
            component_size: value.component_size,
            total_linear_memory_size: value.total_linear_memory_size,
            owned_resources: value
                .owned_resources
                .into_iter()
                .map(|(k, v)| v.try_into().map(|v| (k, v)))
                .collect::<Result<HashMap<_, _>, _>>()?,
            active_plugins: value
                .active_plugins
                .into_iter()
                .map(|id| id.try_into())
                .collect::<Result<HashSet<_>, _>>()?,
            skipped_regions: value
                .skipped_regions
                .into_iter()
                .map(OplogRegion::from)
                .collect::<Vec<_>>(),
            deleted_regions: value
                .deleted_regions
                .into_iter()
                .map(OplogRegion::from)
                .collect::<Vec<_>>(),
        })
    }
}

impl From<WorkerMetadata> for golem_api_grpc::proto::golem::worker::WorkerMetadata {
    fn from(value: WorkerMetadata) -> Self {
        Self {
            worker_id: Some(value.worker_id.into()),
            project_id: Some(value.project_id.into()),
            created_by: Some(value.created_by.into()),
            args: value.args,
            env: value.env,
            wasi_config_vars: Some(value.wasi_config_vars.into()),
            status: value.status.into(),
            component_version: value.component_version,
            retry_count: value.retry_count,
            pending_invocation_count: value.pending_invocation_count,
            updates: value.updates.iter().cloned().map(|u| u.into()).collect(),
            created_at: Some(value.created_at.into()),
            last_error: value.last_error,
            component_size: value.component_size,
            total_linear_memory_size: value.total_linear_memory_size,
            owned_resources: value
                .owned_resources
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            active_plugins: value
                .active_plugins
                .into_iter()
                .map(|id| id.into())
                .collect(),
            skipped_regions: value
                .skipped_regions
                .into_iter()
                .map(|region| region.into())
                .collect(),
            deleted_regions: value
                .deleted_regions
                .into_iter()
                .map(|region| region.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Object)]
pub struct WorkersMetadataResponse {
    pub workers: Vec<WorkerMetadata>,
    pub cursor: Option<ScanCursor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDeploymentRequest {
    pub api_definitions: Vec<ApiDefinitionInfo>,
    pub project_id: ProjectId,
    pub site: ApiSite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDeployment {
    pub api_definitions: Vec<ApiDefinitionInfo>,
    pub project_id: ProjectId,
    pub site: ApiSite,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDefinitionInfo {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
}

impl From<crate::gateway_api_deployment::ApiDeployment> for ApiDeployment {
    fn from(api_deployment: crate::gateway_api_deployment::ApiDeployment) -> Self {
        Self {
            api_definitions: api_deployment
                .api_definition_keys
                .iter()
                .map(|k| ApiDefinitionInfo {
                    id: k.id.clone(),
                    version: k.version.clone(),
                })
                .collect(),
            project_id: api_deployment.namespace.project_id.clone(),
            site: api_deployment.site.clone(),
            created_at: Some(api_deployment.created_at),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDomain {
    pub project_id: ProjectId,
    pub domain_name: String,
    pub name_servers: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiDomain {
    pub fn new(
        request: &DomainRequest,
        name_servers: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            project_id: request.project_id.clone(),
            domain_name: request.domain_name.clone(),
            name_servers,
            created_at: Some(created_at),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountApiDomain {
    pub account_id: AccountId,
    pub domain: ApiDomain,
}

impl AccountApiDomain {
    pub fn new(account_id: &AccountId, domain: &ApiDomain) -> Self {
        Self {
            account_id: account_id.clone(),
            domain: domain.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct DomainRequest {
    pub project_id: ProjectId,
    pub domain_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct CertificateRequest {
    pub project_id: ProjectId,
    pub domain_name: String,
    pub certificate_body: String,
    pub certificate_private_key: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromStr, Serialize, Deserialize, NewType)]
pub struct CertificateId(pub Uuid);

impl Display for CertificateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct Certificate {
    pub id: CertificateId,
    pub project_id: ProjectId,
    pub domain_name: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Certificate {
    pub fn new(request: &CertificateRequest, created_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            id: CertificateId(Uuid::new_v4()),
            project_id: request.project_id.clone(),
            domain_name: request.domain_name.clone(),
            created_at: Some(created_at),
        }
    }
}
