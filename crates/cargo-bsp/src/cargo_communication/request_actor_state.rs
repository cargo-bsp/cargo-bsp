use std::collections::HashMap;

use bsp_types::notifications::TaskId;
use bsp_types::requests::{Request, Run, Test};

use crate::cargo_communication::utils::{generate_random_id, generate_task_id};

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
    pub(super) start_time: i64,
    pub(super) compilation_step: Option<i64>,
}

pub struct RunState {
    pub(super) task_id: TaskId,
}

pub struct TestState {
    pub(super) task_id: TaskId,
    pub(super) suite_task_id: TaskId,
    pub(super) suite_task_progress: SuiteTaskProgress,
    pub(super) single_test_task_ids: HashMap<String, TaskId>,
}

#[derive(Default)]
pub struct SuiteTaskProgress {
    pub(super) progress: i64,
    pub(super) total: i64,
}

impl CompileState {
    pub fn increase_compilation_step(&mut self) {
        self.compilation_step = self.compilation_step.map(|s| s + 1);
    }
}

impl TaskState {
    fn new<R: Request>(root_task_id: TaskId) -> TaskState {
        match R::METHOD {
            Run::METHOD => TaskState::Run(RunState {
                task_id: generate_task_id(&root_task_id),
            }),
            Test::METHOD => {
                let test_task_id = generate_task_id(&root_task_id);
                TaskState::Test(TestState {
                    suite_task_id: generate_task_id(&test_task_id),
                    suite_task_progress: SuiteTaskProgress::default(),
                    task_id: test_task_id,
                    single_test_task_ids: HashMap::new(),
                })
            }
            _ => TaskState::Compile,
        }
    }
}

impl RequestActorState {
    pub fn new<R: Request>(origin_id: Option<String>) -> RequestActorState {
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
            compile_state: CompileState {
                task_id: generate_task_id(&root_task_id),
                ..CompileState::default()
            },
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
