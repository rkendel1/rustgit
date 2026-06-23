use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Repository,
    Service,
    Execution,
    Agent,
    Topology,
    ExecutionImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalIdentity {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub signature: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentityService {
    identities: HashMap<String, GlobalIdentity>,
}

impl IdentityService {
    pub fn register(&mut self, identity: GlobalIdentity) {
        self.identities.insert(identity.entity_id.clone(), identity);
    }

    pub fn get(&self, entity_id: &str) -> Option<&GlobalIdentity> {
        self.identities.get(entity_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStatusSnapshot {
    pub status: AgentStatus,
    pub load: u32,
    pub trust_level: u8,
    pub latency_ms: u32,
}

impl Default for AgentStatusSnapshot {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            load: 0,
            trust_level: 0,
            latency_ms: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ControlPlaneState {
    pub urfs_fingerprints: HashMap<String, RepositoryFingerprint>,
    pub execution_image_specs: HashMap<String, ExecutionImageSpec>,
    pub warm_pool_state: HashMap<String, u32>,
    pub topology_graphs: HashMap<String, ApplicationTopology>,
    pub agent_states: HashMap<String, AgentStatusSnapshot>,
    pub executions: HashMap<String, ExecutionState>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlobalRegistry {
    pub repositories: HashMap<String, RepositoryFingerprint>,
    pub execution_images: HashMap<String, ExecutionImageSpec>,
    pub topologies: HashMap<String, ApplicationTopology>,
    pub executions: HashMap<String, ExecutionState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPolicy {
    PreferDeaWhenTrustedAndCached,
    PreferWarmPoolForColdStart,
    EscalateHeavyWorkloadsToCloud,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityPolicy {
    RejectUnsignedUrfs,
    RejectUntrustedImagesOnDea,
    IsolateExternalProviders,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingPolicy {
    NextJsPrefersNodeWarmPool,
    FastApiPrefersPythonDea,
    MonorepoSplitTopologyFirst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEngine {
    pub execution_policies: Vec<ExecutionPolicy>,
    pub security_policies: Vec<SecurityPolicy>,
    pub routing_policies: Vec<RoutingPolicy>,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self {
            execution_policies: vec![
                ExecutionPolicy::PreferDeaWhenTrustedAndCached,
                ExecutionPolicy::PreferWarmPoolForColdStart,
                ExecutionPolicy::EscalateHeavyWorkloadsToCloud,
            ],
            security_policies: vec![
                SecurityPolicy::RejectUnsignedUrfs,
                SecurityPolicy::RejectUntrustedImagesOnDea,
                SecurityPolicy::IsolateExternalProviders,
            ],
            routing_policies: vec![
                RoutingPolicy::NextJsPrefersNodeWarmPool,
                RoutingPolicy::FastApiPrefersPythonDea,
                RoutingPolicy::MonorepoSplitTopologyFirst,
            ],
        }
    }
}

impl PolicyEngine {
    pub fn allows_urfs(&self, identity: Option<&GlobalIdentity>) -> bool {
        if self
            .security_policies
            .contains(&SecurityPolicy::RejectUnsignedUrfs)
        {
            return identity.is_some_and(|entry| !entry.signature.trim().is_empty());
        }
        true
    }

    pub fn routing_hint(&self, framework_signature: Option<&str>) -> Option<ExecutionTier> {
        let normalized = framework_signature?
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        match normalized.as_str() {
            "nextjs" => Some(ExecutionTier::LocalDocker),
            "fastapi" => Some(ExecutionTier::LocalMachine),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulingContext {
    pub authenticated_identity: bool,
    pub trusted_repo: bool,
    pub cached_runtime: bool,
    pub cold_start_required: bool,
    pub resource_heavy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionScheduler;

impl ExecutionScheduler {
    pub fn schedule(
        &self,
        context: &SchedulingContext,
        policy_engine: &PolicyEngine,
    ) -> ExecutionTier {
        if !context.authenticated_identity {
            return ExecutionTier::ExternalProvider;
        }
        if context.resource_heavy
            && policy_engine
                .execution_policies
                .contains(&ExecutionPolicy::EscalateHeavyWorkloadsToCloud)
        {
            return ExecutionTier::DDockitCloud;
        }
        if context.cold_start_required
            && policy_engine
                .execution_policies
                .contains(&ExecutionPolicy::PreferWarmPoolForColdStart)
        {
            return ExecutionTier::LocalDocker;
        }
        if context.trusted_repo
            && context.cached_runtime
            && policy_engine
                .execution_policies
                .contains(&ExecutionPolicy::PreferDeaWhenTrustedAndCached)
        {
            return ExecutionTier::LocalMachine;
        }
        ExecutionTier::ExternalProvider
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRecord {
    pub identity: AgentIdentity,
    pub capabilities: AgentCapabilities,
    pub status: AgentStatus,
    pub load: u32,
    pub trust_level: u8,
    pub latency_ms: u32,
    pub tier: ExecutionTier,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentRegistry {
    pub agents: HashMap<String, AgentRecord>,
}

impl AgentRegistry {
    pub fn register(&mut self, agent: AgentRecord) {
        self.agents.insert(agent.identity.agent_id.clone(), agent);
    }

    pub fn select_agent(&self, tier: ExecutionTier) -> Option<AgentIdentity> {
        self.agents
            .values()
            .filter(|agent| agent.tier == tier)
            .filter(|agent| !matches!(agent.status, AgentStatus::Offline))
            .max_by(|left, right| {
                left.trust_level
                    .cmp(&right.trust_level)
                    .then_with(|| right.load.cmp(&left.load))
                    .then_with(|| right.latency_ms.cmp(&left.latency_ms))
            })
            .map(|agent| agent.identity.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopologyManager {
    topologies: HashMap<String, ApplicationTopology>,
}

impl TopologyManager {
    pub fn upsert(&mut self, topology: ApplicationTopology) {
        self.topologies
            .insert(topology.topology_id.clone(), topology);
    }

    pub fn get(&self, topology_id: &str) -> Option<&ApplicationTopology> {
        self.topologies.get(topology_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageManager {
    specs: HashMap<String, ExecutionImageSpec>,
}

impl ImageManager {
    pub fn compile_and_store(&mut self, fingerprint: &RepositoryFingerprint) -> ExecutionImageSpec {
        let compiled = ExecutionImageCompiler::compile(fingerprint);
        self.specs
            .insert(fingerprint.repo_id.clone(), compiled.image_spec.clone());
        compiled.image_spec
    }

    pub fn get(&self, repo_id: &str) -> Option<&ExecutionImageSpec> {
        self.specs.get(repo_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Pending,
    Running,
    Failed,
    Migrated,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionState {
    pub urfs: RepositoryFingerprint,
    pub topology: ApplicationTopology,
    pub execution_image: ExecutionImageSpec,
    pub selected_agent: AgentIdentity,
    pub runtime_status: RuntimeState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlPlaneEvent {
    RepositoryAnalyzed {
        repo_id: String,
        fingerprint: RepositoryFingerprint,
    },
    URFSGenerated {
        repo_id: String,
        fingerprint: RepositoryFingerprint,
    },
    ImageCompiled {
        repo_id: String,
        spec: ExecutionImageSpec,
    },
    TopologyBuilt {
        topology_id: String,
        topology: ApplicationTopology,
    },
    AgentRegistered {
        agent_id: String,
        status: AgentStatusSnapshot,
    },
    ExecutionStarted {
        execution_id: String,
        state: ExecutionState,
    },
    ExecutionFailed {
        execution_id: String,
    },
    ExecutionMigrated {
        execution_id: String,
        next_agent: AgentIdentity,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ControlPlaneTelemetry {
    pub execution_latency_global: f64,
    pub routing_accuracy: f32,
    pub dea_utilization: f32,
    pub warm_pool_efficiency: f32,
    pub topology_health_score: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionMigrationEngine;

impl ExecutionMigrationEngine {
    pub fn migrate(state: &mut ExecutionState, next_agent: AgentIdentity) {
        state.selected_agent = next_agent;
        state.runtime_status = RuntimeState::Migrated;
    }
}

pub struct ControlPlane {
    pub identity: IdentityService,
    pub registry: GlobalRegistry,
    pub scheduler: ExecutionScheduler,
    pub topology_manager: TopologyManager,
    pub image_manager: ImageManager,
    pub runtime_router: ExecutionRouter,
    pub agent_manager: AgentRegistry,
    pub policy_engine: PolicyEngine,
    pub state: ControlPlaneState,
    pub telemetry: ControlPlaneTelemetry,
}

impl ControlPlane {
    pub fn new(runtime_router: ExecutionRouter) -> Self {
        Self {
            identity: IdentityService::default(),
            registry: GlobalRegistry::default(),
            scheduler: ExecutionScheduler,
            topology_manager: TopologyManager::default(),
            image_manager: ImageManager::default(),
            runtime_router,
            agent_manager: AgentRegistry::default(),
            policy_engine: PolicyEngine::default(),
            state: ControlPlaneState::default(),
            telemetry: ControlPlaneTelemetry::default(),
        }
    }

    pub fn apply_event(&mut self, event: ControlPlaneEvent) {
        match event {
            ControlPlaneEvent::RepositoryAnalyzed {
                repo_id,
                fingerprint,
            }
            | ControlPlaneEvent::URFSGenerated {
                repo_id,
                fingerprint,
            } => {
                self.state
                    .urfs_fingerprints
                    .insert(repo_id.clone(), fingerprint.clone());
                self.registry.repositories.insert(repo_id, fingerprint);
            }
            ControlPlaneEvent::ImageCompiled { repo_id, spec } => {
                self.state
                    .execution_image_specs
                    .insert(repo_id.clone(), spec.clone());
                self.registry.execution_images.insert(repo_id, spec);
            }
            ControlPlaneEvent::TopologyBuilt {
                topology_id,
                topology,
            } => {
                self.topology_manager.upsert(topology.clone());
                self.state
                    .topology_graphs
                    .insert(topology_id.clone(), topology.clone());
                self.registry.topologies.insert(topology_id, topology);
            }
            ControlPlaneEvent::AgentRegistered { agent_id, status } => {
                self.state.agent_states.insert(agent_id, status);
            }
            ControlPlaneEvent::ExecutionStarted {
                execution_id,
                state,
            } => {
                self.state
                    .executions
                    .insert(execution_id.clone(), state.clone());
                self.registry.executions.insert(execution_id, state);
            }
            ControlPlaneEvent::ExecutionFailed { execution_id } => {
                if let Some(state) = self.state.executions.get_mut(&execution_id) {
                    state.runtime_status = RuntimeState::Failed;
                }
                if let Some(state) = self.registry.executions.get_mut(&execution_id) {
                    state.runtime_status = RuntimeState::Failed;
                }
            }
            ControlPlaneEvent::ExecutionMigrated {
                execution_id,
                next_agent,
            } => {
                if let Some(state) = self.state.executions.get_mut(&execution_id) {
                    ExecutionMigrationEngine::migrate(state, next_agent.clone());
                }
                if let Some(state) = self.registry.executions.get_mut(&execution_id) {
                    ExecutionMigrationEngine::migrate(state, next_agent);
                }
            }
        }
    }
}

pub fn unified_api_routes() -> [&'static str; 5] {
    [
        "POST /execute",
        "GET /state/{execution_id}",
        "POST /migrate/{execution_id}",
        "GET /agents",
        "GET /topology/{id}",
    ]
}
