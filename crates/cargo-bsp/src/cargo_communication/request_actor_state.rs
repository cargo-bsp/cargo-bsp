//! Stores information necessary for [`RequestActor`] that need to be accessible
//! while parsing messages from Cargo command. The information is later used in
//! creation of notifications and responses for the client (especially the state
//! sets and stores TaskIds of all tasks that may potentially be started).

use bsp_types::BuildTargetIdentifier;
use std::collections::HashMap;

use bsp_types::notifications::TaskId;
use bsp_types::requests::{BuildTargetRun, BuildTargetTest, Request};

use crate::cargo_communication::utils::{generate_random_id, generate_task_id, get_current_time};

pub struct RequestActorState {
    pub(super) root_task_id: TaskId,
    pub(super) unit_graph_state: UnitGraphState,
    pub(super) compile_state: CompileState,
    pub(super) task_state: TaskState,
}

pub enum TaskState {
    Compile,
    Run(RunState),
    Test(TestState),
}

pub struct UnitGraphState {
    pub(super) task_id: TaskId,
    pub(super) total_compilation_steps: Option<i64>,
}

#[derive(Default)]
pub struct CompileState {
    pub(super) task_id: TaskId,
    pub(super) errors: i32,
    pub(super) warnings: i32,
    pub(super) compilation_step: Option<i64>,
    pub(super) target_states: HashMap<BuildTargetIdentifier, CompileTargetState>,
}

#[derive(Default)]
pub struct CompileTargetState {
    pub(super) task_id: TaskId,
    pub(super) start_time: i64,
}

pub struct RunState {
    pub(super) task_id: TaskId,
}

#[derive(Default)]
pub struct TestState {
    pub(super) task_id: TaskId,
    pub(super) suite_task_id: TaskId,
    pub(super) suite_task_progress: SuiteTaskProgress,
    /// Currently tested build target.
    pub(super) current_build_target: Option<BuildTargetIdentifier>,
    /// Maps single tests name (by which they are recognized by Cargo) to the TaskId
    /// of the task that they started.
    pub(super) single_test_task_ids: HashMap<String, TaskId>,
}

#[derive(Default)]
pub struct SuiteTaskProgress {
    pub(super) progress: i64,
    pub(super) total: i64,
}

impl CompileState {
    fn new(root_task_id: &TaskId, build_targets: &[BuildTargetIdentifier]) -> CompileState {
        let compile_task_id = generate_task_id(root_task_id);
        let target_states: HashMap<BuildTargetIdentifier, CompileTargetState> = build_targets
            .iter()
            .map(|id| {
                (
                    id.clone(),
                    CompileTargetState {
                        task_id: generate_task_id(&compile_task_id),
                        ..CompileTargetState::default()
                    },
                )
            })
            .collect();
        CompileState {
            task_id: compile_task_id,
            target_states,
            ..CompileState::default()
        }
    }

    pub fn increase_compilation_step(&mut self) {
        self.compilation_step = self.compilation_step.map(|s| s + 1);
    }

    pub fn set_start_time(&mut self, build_target_id: &BuildTargetIdentifier) {
        self.target_states
            .get_mut(build_target_id)
            .unwrap()
            .start_time = get_current_time();
    }

    pub fn get_target_task_id(&self, build_target_id: &BuildTargetIdentifier) -> TaskId {
        self.target_states
            .get(build_target_id)
            .unwrap()
            .task_id
            .clone()
    }
}

impl TaskState {
    fn new<R: Request>(root_task_id: TaskId) -> TaskState {
        match R::METHOD {
            BuildTargetRun::METHOD => TaskState::Run(RunState {
                task_id: generate_task_id(&root_task_id),
            }),
            BuildTargetTest::METHOD => {
                let test_task_id = generate_task_id(&root_task_id);
                TaskState::Test(TestState {
                    suite_task_id: generate_task_id(&test_task_id),
                    task_id: test_task_id,
                    ..TestState::default()
                })
            }
            _ => TaskState::Compile,
        }
    }
}

impl RequestActorState {
    pub fn new<R: Request>(
        origin_id: Option<String>,
        build_targets: &[BuildTargetIdentifier],
    ) -> RequestActorState {
        let root_task_id = TaskId {
            id: origin_id.unwrap_or(generate_random_id()),
            parents: vec![],
        };
        RequestActorState {
            root_task_id: root_task_id.clone(),
            unit_graph_state: UnitGraphState {
                task_id: generate_task_id(&root_task_id),
                total_compilation_steps: None,
            },
            compile_state: CompileState::new(&root_task_id, build_targets),
            task_state: TaskState::new::<R>(root_task_id),
        }
    }

    pub fn get_task_id(&self) -> TaskId {
        match &self.task_state {
            TaskState::Compile => self.root_task_id.clone(),
            TaskState::Run(run_state) => run_state.task_id.clone(),
            TaskState::Test(test_state) => test_state.task_id.clone(),
        }
    }
}
