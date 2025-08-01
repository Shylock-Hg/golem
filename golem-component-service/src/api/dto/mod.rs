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

mod mapper;

use crate::model::plugin as local_plugin_model;
use crate::model::plugin::PluginWasmFileReference;
use golem_common::model::plugin as common_plugin_model;
use golem_common::model::plugin::{PluginOwner, PluginScope};
pub use golem_service_base::dto::{Component, PluginInstallation};
use golem_service_base::poem::TempFileUpload;
use golem_service_base::replayable_stream::ReplayableStream;
pub use mapper::*;
use poem_openapi::types::Binary;
use poem_openapi::Multipart;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, poem_openapi::Union)]
#[oai(discriminator_name = "type", one_of = true)]
#[serde(tag = "type")]
pub enum PluginTypeSpecificCreation {
    ComponentTransformer(common_plugin_model::ComponentTransformerDefinition),
    OplogProcessor(common_plugin_model::OplogProcessorDefinition),
}

impl PluginTypeSpecificCreation {
    pub fn widen(self) -> local_plugin_model::PluginTypeSpecificCreation {
        match self {
            Self::ComponentTransformer(inner) => {
                local_plugin_model::PluginTypeSpecificCreation::ComponentTransformer(inner)
            }
            Self::OplogProcessor(inner) => {
                local_plugin_model::PluginTypeSpecificCreation::OplogProcessor(inner)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, poem_openapi::Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PluginDefinitionCreation {
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: Vec<u8>,
    pub homepage: String,
    pub specs: PluginTypeSpecificCreation,
    pub scope: PluginScope,
}

impl From<PluginDefinitionCreation> for local_plugin_model::PluginDefinitionCreation {
    fn from(value: PluginDefinitionCreation) -> Self {
        local_plugin_model::PluginDefinitionCreation {
            name: value.name,
            version: value.version,
            description: value.description,
            icon: value.icon,
            homepage: value.homepage,
            specs: value.specs.widen(),
            scope: value.scope,
        }
    }
}

#[derive(Multipart)]
#[oai(rename_all = "camelCase")]
pub struct LibraryPluginDefinitionCreation {
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: Binary<Vec<u8>>,
    pub homepage: String,
    pub scope: PluginScope,
    pub wasm: TempFileUpload,
}

impl From<LibraryPluginDefinitionCreation> for local_plugin_model::PluginDefinitionCreation {
    fn from(value: LibraryPluginDefinitionCreation) -> Self {
        local_plugin_model::PluginDefinitionCreation {
            name: value.name,
            version: value.version,
            description: value.description,
            icon: value.icon.0,
            homepage: value.homepage,
            specs: local_plugin_model::PluginTypeSpecificCreation::Library(
                local_plugin_model::LibraryPluginCreation {
                    data: PluginWasmFileReference::Data(value.wasm.boxed()),
                },
            ),
            scope: value.scope,
        }
    }
}

#[derive(Multipart)]
#[oai(rename_all = "camelCase")]
pub struct AppPluginDefinitionCreation {
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: Binary<Vec<u8>>,
    pub homepage: String,
    pub scope: PluginScope,
    pub wasm: TempFileUpload,
}

impl From<AppPluginDefinitionCreation> for local_plugin_model::PluginDefinitionCreation {
    fn from(value: AppPluginDefinitionCreation) -> Self {
        local_plugin_model::PluginDefinitionCreation {
            name: value.name,
            version: value.version,
            description: value.description,
            icon: value.icon.0,
            homepage: value.homepage,
            specs: local_plugin_model::PluginTypeSpecificCreation::App(
                local_plugin_model::AppPluginCreation {
                    data: PluginWasmFileReference::Data(value.wasm.boxed()),
                },
            ),
            scope: value.scope,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, poem_openapi::Union)]
#[oai(discriminator_name = "type", one_of = true)]
#[serde(tag = "type")]
pub enum PluginTypeSpecificDefinition {
    ComponentTransformer(common_plugin_model::ComponentTransformerDefinition),
    OplogProcessor(common_plugin_model::OplogProcessorDefinition),
    Library(LibraryPluginDefinition),
    App(AppPluginDefinition),
}

impl From<common_plugin_model::PluginTypeSpecificDefinition> for PluginTypeSpecificDefinition {
    fn from(value: common_plugin_model::PluginTypeSpecificDefinition) -> Self {
        match value {
            common_plugin_model::PluginTypeSpecificDefinition::ComponentTransformer(value) => {
                Self::ComponentTransformer(value)
            }
            common_plugin_model::PluginTypeSpecificDefinition::OplogProcessor(value) => {
                Self::OplogProcessor(value)
            }
            common_plugin_model::PluginTypeSpecificDefinition::App(value) => {
                Self::App(value.into())
            }
            common_plugin_model::PluginTypeSpecificDefinition::Library(value) => {
                Self::Library(value.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, poem_openapi::Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LibraryPluginDefinition {}

impl From<common_plugin_model::LibraryPluginDefinition> for LibraryPluginDefinition {
    fn from(_value: common_plugin_model::LibraryPluginDefinition) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, poem_openapi::Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AppPluginDefinition {}

impl From<common_plugin_model::AppPluginDefinition> for AppPluginDefinition {
    fn from(_value: common_plugin_model::AppPluginDefinition) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, poem_openapi::Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PluginDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: Vec<u8>,
    pub homepage: String,
    pub specs: PluginTypeSpecificDefinition,
    pub scope: PluginScope,
    pub owner: PluginOwner,
}

impl From<common_plugin_model::PluginDefinition> for PluginDefinition {
    fn from(value: common_plugin_model::PluginDefinition) -> Self {
        Self {
            name: value.name,
            version: value.version,
            description: value.description,
            icon: value.icon,
            homepage: value.homepage,
            specs: value.specs.into(),
            scope: value.scope,
            owner: value.owner,
        }
    }
}
