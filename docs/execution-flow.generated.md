# Execution Flow (Generated, Code-Grounded)

## Entry points
- ExecutionEngine::start
- WorkspaceManager::launch
- WorkspaceManager::restart
- WorkspaceManager::stop

## Runtime behavior (derived from call paths)
- ArtifactStore is checked for existing outputs
- CacheKeyEngine computes node keys
- ExecutionGraph is generated via BuildPlanner
- ExecutionRouter selects runtime ownership
- Provider executes node
- RepositoryAnalysis is produced by repository analyzer
- Result is stored in ArtifactStore

## Workspace state machine transitions (actual transitions only)
- Analyzing -> Failed
- Analyzing -> Planning
- Created -> Materializing
- Degraded -> Failed
- Degraded -> Migrating
- Degraded -> Restarting
- Degraded -> Running
- Failed -> Destroyed
- Failed -> Migrating
- Failed -> Restarting
- Failed -> Starting
- Failed -> Stopped
- Failed -> Stopping
- Initializing -> Failed
- Initializing -> Ready
- Initializing -> Running
- Installing -> Analyzing
- Installing -> Failed
- Installing -> Launching
- Launching -> Failed
- Launching -> Initializing
- Materializing -> Analyzing
- Materializing -> Failed
- Materializing -> Installing
- Migrating -> Failed
- Migrating -> Running
- Migrating -> Starting
- Paused -> Failed
- Paused -> Running
- Paused -> Stopping
- Pending -> Failed
- Pending -> Provisioning
- Planning -> Failed
- Planning -> Launching
- Planning -> Starting
- Provisioning -> Failed
- Provisioning -> Starting
- Ready -> Failed
- Ready -> Running
- Ready -> Stopping
- Restarting -> Failed
- Restarting -> Running
- Restarting -> Starting
- Running -> Degraded
- Running -> Failed
- Running -> Migrating
- Running -> Paused
- Running -> Restarting
- Running -> Stopping
- Starting -> Failed
- Starting -> Launching
- Starting -> Running
- Stopped -> Destroyed
- Stopped -> Provisioning
- Stopped -> Restarting
- Stopped -> Starting
- Stopping -> Failed
- Stopping -> Stopped

If a transition or call is not listed above, it was not extracted from current code.