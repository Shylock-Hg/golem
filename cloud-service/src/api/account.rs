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

use super::dto;
use crate::api::ApiResult;
use crate::model::*;
use crate::service::account::{AccountError, AccountService};
use crate::service::auth::AuthService;
use golem_common::model::auth::AccountAction;
use golem_common::model::AccountId;
use golem_common::recorded_http_api_request;
use golem_service_base::api_tags::ApiTags;
use golem_service_base::model::auth::GolemSecurityScheme;
use param::Query;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::*;
use std::sync::Arc;
use tracing::Instrument;

pub struct AccountApi {
    pub auth_service: Arc<dyn AuthService>,
    pub account_service: Arc<dyn AccountService>,
}

#[OpenApi(prefix_path = "/v1/accounts", tag = ApiTags::Account)]
impl AccountApi {
    /// Find accounts
    ///
    /// Find matching accounts. Only your own account or accounts you have at least one grant from will be returned
    #[oai(path = "/", method = "get", operation_id = "find_accounts")]
    async fn find_accounts(
        &self,
        email: Query<Option<String>>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<dto::FindAccountsResponse>> {
        let record = recorded_http_api_request!("find_accounts", email = email.0);
        let response = self
            .find_accounts_internal(email.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn find_accounts_internal(
        &self,
        email: Option<String>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<dto::FindAccountsResponse>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        let viewable_accounts = self.auth_service.viewable_accounts(&auth).await?;

        let values = self
            .account_service
            .find(email.as_deref(), viewable_accounts)
            .await?;
        Ok(Json(dto::FindAccountsResponse { values }))
    }

    /// Get account
    ///
    /// Retrieve an account for a given Account ID
    #[oai(path = "/:account_id", method = "get", operation_id = "get_account")]
    async fn get_account(
        &self,
        account_id: Path<AccountId>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let record =
            recorded_http_api_request!("get_account", account_id = account_id.0.to_string());
        let response = self
            .get_account_internal(account_id.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn get_account_internal(
        &self,
        account_id: AccountId,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        self.auth_service
            .authorize_account_action(&auth, &account_id, &AccountAction::ViewAccount)
            .await?;

        let response = self.account_service.get(&account_id).await?;
        Ok(Json(response))
    }

    /// Get account's plan
    #[oai(
        path = "/:account_id/plan",
        method = "get",
        operation_id = "get_account_plan"
    )]
    async fn get_account_plan(
        &self,
        account_id: Path<AccountId>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Plan>> {
        let record =
            recorded_http_api_request!("get_account_plan", account_id = account_id.0.to_string());
        let response = self
            .get_account_plan_internal(account_id.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn get_account_plan_internal(
        &self,
        account_id: AccountId,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Plan>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        self.auth_service
            .authorize_account_action(&auth, &account_id, &AccountAction::ViewPlan)
            .await?;

        let response = self.account_service.get_plan(&account_id).await?;
        Ok(Json(response))
    }

    /// Update account
    ///
    /// Allows the user to change the account details such as name and email.
    ///
    /// Changing the planId is not allowed and the request will be rejected.
    /// The response is the updated account data.
    #[oai(path = "/:account_id", method = "put", operation_id = "update_account")]
    async fn put_account(
        &self,
        account_id: Path<AccountId>,
        data: Json<AccountData>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let record =
            recorded_http_api_request!("update_account", account_id = account_id.0.to_string());
        let response = self
            .put_account_internal(account_id.0, data.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn put_account_internal(
        &self,
        account_id: AccountId,
        data: AccountData,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        self.auth_service
            .authorize_account_action(&auth, &account_id, &AccountAction::UpdateAccount)
            .await?;

        let response = self.account_service.update(&account_id, &data).await?;
        Ok(Json(response))
    }

    /// Create account
    ///
    /// Create a new account. The response is the created account data.
    #[oai(path = "/", method = "post", operation_id = "create_account")]
    async fn post_account(
        &self,
        data: Json<AccountData>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let record = recorded_http_api_request!("create_account", account_name = data.name.clone());
        let response = self
            .post_account_internal(data.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn post_account_internal(
        &self,
        data: AccountData,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<Account>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        self.auth_service
            .authorize_global_action(&auth, &GlobalAction::CreateAccount)
            .await?;

        let response = self
            .account_service
            .create(&AccountId::generate(), &data)
            .await?;
        Ok(Json(response))
    }

    /// Delete account
    ///
    /// Delete an account.
    #[oai(
        path = "/:account_id",
        method = "delete",
        operation_id = "delete_account"
    )]
    async fn delete_account(
        &self,
        account_id: Path<AccountId>,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<DeleteAccountResponse>> {
        let record =
            recorded_http_api_request!("delete_account", account_id = account_id.0.to_string());
        let response = self
            .delete_account_internal(account_id.0, token)
            .instrument(record.span.clone())
            .await;

        record.result(response)
    }

    async fn delete_account_internal(
        &self,
        account_id: AccountId,
        token: GolemSecurityScheme,
    ) -> ApiResult<Json<DeleteAccountResponse>> {
        let auth = self.auth_service.authorization(token.as_ref()).await?;
        self.auth_service
            .authorize_account_action(&auth, &account_id, &AccountAction::DeleteAccount)
            .await?;

        if auth.token.account_id == account_id {
            Err(AccountError::ArgValidation(vec![
                "Cannot delete current account.".to_string(),
            ]))?;
        };

        self.account_service.delete(&account_id).await?;
        Ok(Json(DeleteAccountResponse {}))
    }
}
