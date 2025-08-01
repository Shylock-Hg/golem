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

use super::file_loader::FileLoader;
use crate::durable_host::serialized::SerializableError;
use crate::metrics::workers::record_worker_call;
use crate::model::ExecutionStatus;
use crate::preview2::golem_api_1_x::host::ForkResult;
use crate::services::events::Events;
use crate::services::oplog::plugin::OplogProcessorPlugin;
use crate::services::oplog::{CommitLevel, Oplog, OplogOps};
use crate::services::plugins::Plugins;
use crate::services::projects::ProjectService;
use crate::services::resource_limits::ResourceLimits;
use crate::services::rpc::Rpc;
use crate::services::shard::ShardService;
use crate::services::worker_proxy::WorkerProxy;
use crate::services::{
    active_workers, blob_store, component, golem_config, key_value, oplog, promise, scheduler,
    shard_manager, worker, worker_activator, worker_enumeration, HasActiveWorkers,
    HasBlobStoreService, HasComponentService, HasConfig, HasEvents, HasExtraDeps, HasFileLoader,
    HasKeyValueService, HasOplogProcessorPlugin, HasOplogService, HasPlugins, HasProjectService,
    HasPromiseService, HasResourceLimits, HasRpc, HasRunningWorkerEnumerationService,
    HasSchedulerService, HasShardManagerService, HasShardService, HasWasmtimeEngine,
    HasWorkerActivator, HasWorkerEnumerationService, HasWorkerProxy, HasWorkerService,
};
use crate::services::{rdbms, HasOplog, HasRdbmsService, HasWorkerForkService};
use crate::worker::Worker;
use crate::workerctx::WorkerCtx;
use async_trait::async_trait;
use golem_common::model::oplog::{DurableFunctionType, OplogIndex, OplogIndexRange};
use golem_common::model::{AccountId, ProjectId, Timestamp, WorkerMetadata, WorkerStatusRecord};
use golem_common::model::{OwnedWorkerId, WorkerId};
use golem_common::serialization::serialize;
use golem_service_base::error::worker_executor::WorkerExecutorError;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::runtime::Handle;

#[async_trait]
pub trait WorkerForkService: Send + Sync {
    async fn fork(
        &self,
        fork_account_id: &AccountId,
        source_worker_id: &OwnedWorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<(), WorkerExecutorError>;

    async fn fork_and_write_fork_result(
        &self,
        fork_account_id: &AccountId,
        source_worker_id: &OwnedWorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<(), WorkerExecutorError>;
}

pub struct DefaultWorkerFork<Ctx: WorkerCtx> {
    pub rpc: Arc<dyn Rpc>,
    pub active_workers: Arc<active_workers::ActiveWorkers<Ctx>>,
    pub engine: Arc<wasmtime::Engine>,
    pub linker: Arc<wasmtime::component::Linker<Ctx>>,
    pub runtime: Handle,
    pub component_service: Arc<dyn component::ComponentService>,
    pub shard_manager_service: Arc<dyn shard_manager::ShardManagerService>,
    pub worker_service: Arc<dyn worker::WorkerService>,
    pub worker_proxy: Arc<dyn WorkerProxy>,
    pub worker_enumeration_service: Arc<dyn worker_enumeration::WorkerEnumerationService>,
    pub running_worker_enumeration_service:
        Arc<dyn worker_enumeration::RunningWorkerEnumerationService>,
    pub promise_service: Arc<dyn promise::PromiseService>,
    pub golem_config: Arc<golem_config::GolemConfig>,
    pub shard_service: Arc<dyn ShardService>,
    pub key_value_service: Arc<dyn key_value::KeyValueService>,
    pub blob_store_service: Arc<dyn blob_store::BlobStoreService>,
    pub rdbms_service: Arc<dyn rdbms::RdbmsService>,
    pub oplog_service: Arc<dyn oplog::OplogService>,
    pub scheduler_service: Arc<dyn scheduler::SchedulerService>,
    pub worker_activator: Arc<dyn worker_activator::WorkerActivator<Ctx>>,
    pub events: Arc<Events>,
    pub file_loader: Arc<FileLoader>,
    pub plugins: Arc<dyn Plugins>,
    pub oplog_processor_plugin: Arc<dyn OplogProcessorPlugin>,
    pub resource_limits: Arc<dyn ResourceLimits>,
    pub project_service: Arc<dyn ProjectService>,
    pub extra_deps: Ctx::ExtraDeps,
}

impl<Ctx: WorkerCtx> HasEvents for DefaultWorkerFork<Ctx> {
    fn events(&self) -> Arc<Events> {
        self.events.clone()
    }
}

impl<Ctx: WorkerCtx> HasActiveWorkers<Ctx> for DefaultWorkerFork<Ctx> {
    fn active_workers(&self) -> Arc<active_workers::ActiveWorkers<Ctx>> {
        self.active_workers.clone()
    }
}

impl<Ctx: WorkerCtx> HasComponentService for DefaultWorkerFork<Ctx> {
    fn component_service(&self) -> Arc<dyn component::ComponentService> {
        self.component_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasConfig for DefaultWorkerFork<Ctx> {
    fn config(&self) -> Arc<golem_config::GolemConfig> {
        self.golem_config.clone()
    }
}

impl<Ctx: WorkerCtx> HasWorkerService for DefaultWorkerFork<Ctx> {
    fn worker_service(&self) -> Arc<dyn worker::WorkerService> {
        self.worker_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasWorkerEnumerationService for DefaultWorkerFork<Ctx> {
    fn worker_enumeration_service(&self) -> Arc<dyn worker_enumeration::WorkerEnumerationService> {
        self.worker_enumeration_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasRunningWorkerEnumerationService for DefaultWorkerFork<Ctx> {
    fn running_worker_enumeration_service(
        &self,
    ) -> Arc<dyn worker_enumeration::RunningWorkerEnumerationService> {
        self.running_worker_enumeration_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasPromiseService for DefaultWorkerFork<Ctx> {
    fn promise_service(&self) -> Arc<dyn promise::PromiseService> {
        self.promise_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasWasmtimeEngine<Ctx> for DefaultWorkerFork<Ctx> {
    fn engine(&self) -> Arc<wasmtime::Engine> {
        self.engine.clone()
    }

    fn linker(&self) -> Arc<wasmtime::component::Linker<Ctx>> {
        self.linker.clone()
    }

    fn runtime(&self) -> Handle {
        self.runtime.clone()
    }
}

impl<Ctx: WorkerCtx> HasKeyValueService for DefaultWorkerFork<Ctx> {
    fn key_value_service(&self) -> Arc<dyn key_value::KeyValueService> {
        self.key_value_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasRdbmsService for DefaultWorkerFork<Ctx> {
    fn rdbms_service(&self) -> Arc<dyn rdbms::RdbmsService> {
        self.rdbms_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasBlobStoreService for DefaultWorkerFork<Ctx> {
    fn blob_store_service(&self) -> Arc<dyn blob_store::BlobStoreService> {
        self.blob_store_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasSchedulerService for DefaultWorkerFork<Ctx> {
    fn scheduler_service(&self) -> Arc<dyn scheduler::SchedulerService> {
        self.scheduler_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasOplogService for DefaultWorkerFork<Ctx> {
    fn oplog_service(&self) -> Arc<dyn oplog::OplogService> {
        self.oplog_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasWorkerForkService for DefaultWorkerFork<Ctx> {
    fn worker_fork_service(&self) -> Arc<dyn WorkerForkService> {
        Arc::new(self.clone())
    }
}

impl<Ctx: WorkerCtx> HasRpc for DefaultWorkerFork<Ctx> {
    fn rpc(&self) -> Arc<dyn Rpc> {
        self.rpc.clone()
    }
}

impl<Ctx: WorkerCtx> HasExtraDeps<Ctx> for DefaultWorkerFork<Ctx> {
    fn extra_deps(&self) -> Ctx::ExtraDeps {
        self.extra_deps.clone()
    }
}

impl<Ctx: WorkerCtx> HasShardService for DefaultWorkerFork<Ctx> {
    fn shard_service(&self) -> Arc<dyn ShardService> {
        self.shard_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasShardManagerService for DefaultWorkerFork<Ctx> {
    fn shard_manager_service(&self) -> Arc<dyn shard_manager::ShardManagerService> {
        self.shard_manager_service.clone()
    }
}

impl<Ctx: WorkerCtx> HasWorkerActivator<Ctx> for DefaultWorkerFork<Ctx> {
    fn worker_activator(&self) -> Arc<dyn worker_activator::WorkerActivator<Ctx>> {
        self.worker_activator.clone()
    }
}

impl<Ctx: WorkerCtx> HasWorkerProxy for DefaultWorkerFork<Ctx> {
    fn worker_proxy(&self) -> Arc<dyn WorkerProxy> {
        self.worker_proxy.clone()
    }
}

impl<Ctx: WorkerCtx> HasFileLoader for DefaultWorkerFork<Ctx> {
    fn file_loader(&self) -> Arc<FileLoader> {
        self.file_loader.clone()
    }
}

impl<Ctx: WorkerCtx> HasPlugins for DefaultWorkerFork<Ctx> {
    fn plugins(&self) -> Arc<dyn Plugins> {
        self.plugins.clone()
    }
}

impl<Ctx: WorkerCtx> HasOplogProcessorPlugin for DefaultWorkerFork<Ctx> {
    fn oplog_processor_plugin(&self) -> Arc<dyn OplogProcessorPlugin> {
        self.oplog_processor_plugin.clone()
    }
}

impl<Ctx: WorkerCtx> HasResourceLimits for DefaultWorkerFork<Ctx> {
    fn resource_limits(&self) -> Arc<dyn ResourceLimits> {
        self.resource_limits.clone()
    }
}

impl<Ctx: WorkerCtx> HasProjectService for DefaultWorkerFork<Ctx> {
    fn project_service(&self) -> Arc<dyn ProjectService> {
        self.project_service.clone()
    }
}

impl<Ctx: WorkerCtx> Clone for DefaultWorkerFork<Ctx> {
    fn clone(&self) -> Self {
        Self {
            rpc: self.rpc.clone(),
            active_workers: self.active_workers.clone(),
            engine: self.engine.clone(),
            linker: self.linker.clone(),
            runtime: self.runtime.clone(),
            component_service: self.component_service.clone(),
            shard_manager_service: self.shard_manager_service.clone(),
            worker_service: self.worker_service.clone(),
            worker_proxy: self.worker_proxy.clone(),
            worker_enumeration_service: self.worker_enumeration_service.clone(),
            running_worker_enumeration_service: self.running_worker_enumeration_service.clone(),
            promise_service: self.promise_service.clone(),
            golem_config: self.golem_config.clone(),
            shard_service: self.shard_service.clone(),
            key_value_service: self.key_value_service.clone(),
            blob_store_service: self.blob_store_service.clone(),
            rdbms_service: self.rdbms_service.clone(),
            oplog_service: self.oplog_service.clone(),
            scheduler_service: self.scheduler_service.clone(),
            worker_activator: self.worker_activator.clone(),
            events: self.events.clone(),
            file_loader: self.file_loader.clone(),
            plugins: self.plugins.clone(),
            oplog_processor_plugin: self.oplog_processor_plugin.clone(),
            resource_limits: self.resource_limits.clone(),
            project_service: self.project_service.clone(),
            extra_deps: self.extra_deps.clone(),
        }
    }
}

impl<Ctx: WorkerCtx> DefaultWorkerFork<Ctx> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rpc: Arc<dyn Rpc>,
        active_workers: Arc<active_workers::ActiveWorkers<Ctx>>,
        engine: Arc<wasmtime::Engine>,
        linker: Arc<wasmtime::component::Linker<Ctx>>,
        runtime: Handle,
        component_service: Arc<dyn component::ComponentService>,
        shard_manager_service: Arc<dyn shard_manager::ShardManagerService>,
        worker_service: Arc<dyn worker::WorkerService>,
        worker_proxy: Arc<dyn WorkerProxy>,
        worker_enumeration_service: Arc<dyn worker_enumeration::WorkerEnumerationService>,
        running_worker_enumeration_service: Arc<
            dyn worker_enumeration::RunningWorkerEnumerationService,
        >,
        promise_service: Arc<dyn promise::PromiseService>,
        golem_config: Arc<golem_config::GolemConfig>,
        shard_service: Arc<dyn ShardService>,
        key_value_service: Arc<dyn key_value::KeyValueService>,
        blob_store_service: Arc<dyn blob_store::BlobStoreService>,
        rdbms_service: Arc<dyn rdbms::RdbmsService>,
        oplog_service: Arc<dyn oplog::OplogService>,
        scheduler_service: Arc<dyn scheduler::SchedulerService>,
        worker_activator: Arc<dyn worker_activator::WorkerActivator<Ctx>>,
        events: Arc<Events>,
        file_loader: Arc<FileLoader>,
        plugins: Arc<dyn Plugins>,
        oplog_processor_plugin: Arc<dyn OplogProcessorPlugin>,
        resource_limits: Arc<dyn ResourceLimits>,
        project_service: Arc<dyn ProjectService>,
        extra_deps: Ctx::ExtraDeps,
    ) -> Self {
        Self {
            rpc,
            active_workers,
            engine,
            linker,
            runtime,
            component_service,
            shard_manager_service,
            worker_service,
            worker_proxy,
            worker_enumeration_service,
            running_worker_enumeration_service,
            promise_service,
            golem_config,
            shard_service,
            key_value_service,
            blob_store_service,
            rdbms_service,
            oplog_service,
            scheduler_service,
            worker_activator,
            events,
            file_loader,
            plugins,
            oplog_processor_plugin,
            resource_limits,
            project_service,
            extra_deps,
        }
    }

    async fn validate_worker_forking(
        &self,
        project_id: &ProjectId,
        source_worker_id: &WorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<(OwnedWorkerId, OwnedWorkerId), WorkerExecutorError> {
        let second_index = OplogIndex::INITIAL.next();

        if oplog_index_cut_off < second_index {
            return Err(WorkerExecutorError::invalid_request(
                "oplog_index_cut_off must be at least 2",
            ));
        }

        let owned_target_worker_id = OwnedWorkerId::new(project_id, target_worker_id);

        let target_metadata = self.worker_service.get(&owned_target_worker_id).await;

        // We allow forking only if the target worker does not exist
        if target_metadata.is_some() {
            return Err(WorkerExecutorError::worker_already_exists(
                target_worker_id.clone(),
            ));
        }

        // We assume the source worker belongs to this executor
        self.shard_service.check_worker(source_worker_id)?;

        let owned_source_worker_id = OwnedWorkerId::new(project_id, source_worker_id);

        self.worker_service
            .get(&owned_source_worker_id)
            .await
            .ok_or(WorkerExecutorError::worker_not_found(
                source_worker_id.clone(),
            ))?;

        Ok((owned_source_worker_id, owned_target_worker_id))
    }

    async fn copy_source_oplog(
        &self,
        fork_account_id: &AccountId,
        source_worker_id: &OwnedWorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<Arc<dyn Oplog>, WorkerExecutorError> {
        record_worker_call("fork");

        let (owned_source_worker_id, owned_target_worker_id) = self
            .validate_worker_forking(
                &source_worker_id.project_id,
                &source_worker_id.worker_id,
                target_worker_id,
                oplog_index_cut_off,
            )
            .await?;

        let target_worker_id = owned_target_worker_id.worker_id.clone();
        let project_id = owned_target_worker_id.project_id.clone();

        let source_worker_instance = Worker::get_or_create_suspended(
            self,
            fork_account_id,
            &owned_source_worker_id,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        let source_worker_metadata = source_worker_instance.get_metadata()?;

        let target_worker_metadata = WorkerMetadata {
            worker_id: target_worker_id.clone(),
            created_by: fork_account_id.clone(),
            project_id,
            env: source_worker_metadata.env.clone(),
            args: source_worker_metadata.args.clone(),
            wasi_config_vars: source_worker_metadata.wasi_config_vars.clone(),
            created_at: Timestamp::now_utc(),
            parent: None,
            last_known_status: WorkerStatusRecord::default(),
        };

        let source_oplog = source_worker_instance.oplog();

        source_oplog.commit(CommitLevel::Always).await;

        let initial_oplog_entry = source_oplog.read(OplogIndex::INITIAL).await;

        // Update the oplog initial entry with the new worker
        let target_initial_oplog_entry = initial_oplog_entry
            .update_worker_id(&target_worker_id)
            .ok_or(WorkerExecutorError::unknown(
                "Failed to update worker id in oplog entry",
            ))?;

        let new_oplog = self
            .oplog_service
            .create(
                &owned_target_worker_id,
                target_initial_oplog_entry,
                target_worker_metadata,
                Arc::new(RwLock::new(ExecutionStatus::Suspended {
                    last_known_status: WorkerStatusRecord::default(),
                    component_type: source_worker_instance.component_type(),
                    timestamp: Timestamp::now_utc(),
                })),
            )
            .await;

        let oplog_range = OplogIndexRange::new(OplogIndex::INITIAL.next(), oplog_index_cut_off);

        for oplog_index in oplog_range {
            let entry = source_oplog.read(oplog_index).await;
            new_oplog.add(entry.clone()).await;
        }

        Ok(new_oplog)
    }
}

#[async_trait]
impl<Ctx: WorkerCtx> WorkerForkService for DefaultWorkerFork<Ctx> {
    async fn fork(
        &self,
        fork_account_id: &AccountId,
        source_worker_id: &OwnedWorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<(), WorkerExecutorError> {
        let new_oplog = self
            .copy_source_oplog(
                fork_account_id,
                source_worker_id,
                target_worker_id,
                oplog_index_cut_off,
            )
            .await?;

        new_oplog.commit(CommitLevel::Always).await;

        // We go through worker proxy to resume the worker
        // as we need to make sure as it may live in another worker executor,
        // depending on sharding.
        // This will replay until the fork point in the forked worker
        self.worker_proxy
            .resume(target_worker_id, true)
            .await
            .map_err(|err| {
                WorkerExecutorError::failed_to_resume_worker(target_worker_id.clone(), err.into())
            })?;

        Ok(())
    }

    async fn fork_and_write_fork_result(
        &self,
        fork_account_id: &AccountId,
        source_worker_id: &OwnedWorkerId,
        target_worker_id: &WorkerId,
        oplog_index_cut_off: OplogIndex,
    ) -> Result<(), WorkerExecutorError> {
        let new_oplog = self
            .copy_source_oplog(
                fork_account_id,
                source_worker_id,
                target_worker_id,
                oplog_index_cut_off,
            )
            .await?;

        // durability.persist will write an ImportedFunctionInvoked entry persisting ForkResult::Original
        // we write an alternative version of that entry to the new oplog, so it is going to return with
        // ForkResult::Forked in the other worker
        let serialized_input = serialize(&target_worker_id.worker_name).map_err(|err| {
            WorkerExecutorError::runtime(format!("failed to serialize worker name for persisting durable function invocation: {err}"))
        })?.to_vec();

        let forked: Result<ForkResult, SerializableError> = Ok(ForkResult::Forked);
        let serialized_response = serialize(&forked).map_err(|err| {
            WorkerExecutorError::runtime(format!("failed to serialize fork result for persisting durable function invocation: {err}"))
        })?.to_vec();

        let _ = new_oplog
            .add_raw_imported_function_invoked(
                "golem::api::fork".to_string(),
                &serialized_input,
                &serialized_response,
                DurableFunctionType::WriteRemote,
            )
            .await
            .map_err(|err| {
                WorkerExecutorError::runtime(format!(
                    "failed to serialize and store durable function invocation: {err}"
                ))
            });

        new_oplog.commit(CommitLevel::Always).await;

        // We go through worker proxy to resume the worker
        // as we need to make sure as it may live in another worker executor,
        // depending on sharding.
        // This will replay until the fork point in the forked worker
        self.worker_proxy
            .resume(target_worker_id, true)
            .await
            .map_err(|err| {
                WorkerExecutorError::failed_to_resume_worker(target_worker_id.clone(), err.into())
            })?;

        Ok(())
    }
}
