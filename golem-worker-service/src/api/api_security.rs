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

use super::dto::SecuritySchemeData;
use crate::api::common::ApiEndpointError;
use crate::gateway_security::{SecurityScheme, SecuritySchemeIdentifier};
use crate::service::api_security::SecuritySchemeService;
use golem_common::model::auth::AuthCtx;
use golem_common::model::ProjectId;
use golem_common::{recorded_http_api_request, safe};
use golem_service_base::api_tags::ApiTags;
use golem_service_base::model::auth::GolemSecurityScheme;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use std::sync::Arc;
use tracing::Instrument;

pub struct SecuritySchemeApi {
    security_scheme_service: Arc<dyn SecuritySchemeService>,
}

impl SecuritySchemeApi {
    pub fn new(security_scheme_service: Arc<dyn SecuritySchemeService>) -> Self {
        Self {
            security_scheme_service,
        }
    }
}

#[OpenApi(prefix_path = "/v1/api/security",  tag = ApiTags::ApiSecurity)]
impl SecuritySchemeApi {
    /// Get a security scheme
    ///
    /// Get a security scheme by name
    #[oai(
        path = "/:project_id/:security_scheme_identifier",
        method = "get",
        operation_id = "get"
    )]
    async fn get(
        &self,
        project_id: Path<ProjectId>,
        token: GolemSecurityScheme,
        security_scheme_identifier: Path<String>,
    ) -> Result<Json<SecuritySchemeData>, ApiEndpointError> {
        let token = token.secret();
        let project_id = project_id.0;

        let record = recorded_http_api_request!(
            "get",
            security_scheme_identifier = security_scheme_identifier.0
        );
        let security_scheme = self
            .security_scheme_service
            .get(
                &SecuritySchemeIdentifier::new(security_scheme_identifier.0),
                &project_id,
                &AuthCtx::new(token),
            )
            .instrument(record.span.clone())
            .await?;

        Ok(Json(SecuritySchemeData::from(security_scheme)))
    }

    /// Create a security scheme
    #[oai(path = "/:project_id", method = "post", operation_id = "create")]
    async fn create(
        &self,
        project_id: Path<ProjectId>,
        payload: Json<SecuritySchemeData>,
        token: GolemSecurityScheme,
    ) -> Result<Json<SecuritySchemeData>, ApiEndpointError> {
        let record = recorded_http_api_request!(
            "create",
            security_scheme_identifier = payload.0.scheme_identifier
        );

        let response = self
            .create_internal(project_id.0, payload.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn create_internal(
        &self,
        project_id: ProjectId,
        payload: SecuritySchemeData,
        token: GolemSecurityScheme,
    ) -> Result<Json<SecuritySchemeData>, ApiEndpointError> {
        let token = token.secret();
        let security_scheme = SecurityScheme::try_from(payload).map_err(|err| {
            ApiEndpointError::bad_request(safe(format!("Invalid security scheme {err}")))
        })?;

        let security_scheme_with_metadata = self
            .security_scheme_service
            .create(&security_scheme, &project_id, &AuthCtx::new(token))
            .await?;

        Ok(Json(SecuritySchemeData::from(
            security_scheme_with_metadata,
        )))
    }
}
