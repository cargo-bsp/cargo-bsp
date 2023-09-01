//! Implementation of [`RequestActor`]. Parses messages from Cargo, handles them
//! and creates the appropriate notifications for the client.

use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::{BuildFinished, CompilerMessage, Message};
use log::warn;
use lsp_types::DiagnosticSeverity;
use path_absolutize::*;
use paths::AbsPath;

use bsp_types::notifications::{
    CompileReportData, LogMessageParams, MessageType, OnBuildLogMessage, OnBuildPublishDiagnostics,
    PublishDiagnosticsParams, TaskDataWithKind, TaskId, TestStartData, TestStatus, TestTaskData,
};
use bsp_types::requests::Request;
use bsp_types::StatusCode;

use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::CargoMessage;
use crate::cargo_communication::cargo_types::params_target::ParamsTarget;
use crate::cargo_communication::cargo_types::publish_diagnostics::{
    map_cargo_diagnostic_to_bsp, DiagnosticMessage, GlobalMessage,
};
use crate::cargo_communication::cargo_types::test::{
    SuiteEvent, SuiteResults, TestEvent, TestResult, TestType,
};
use crate::cargo_communication::request_actor::{CargoHandler, RequestActor};
use crate::cargo_communication::request_actor_state::{SuiteTaskProgress, TaskState};
use crate::cargo_communication::utils::{generate_random_id, generate_task_id, get_current_time};

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand + ParamsTarget,
    R::Result: CargoResult,
    C: CargoHandler<CargoMessage>,
{
    pub(super) fn handle_cargo_information(&mut self, message: Message) {
        match message {
            Message::CompilerArtifact(msg) => {
                self.report_compile_step(serde_json::to_string(&msg).ok());
            }
            Message::CompilerMessage(msg) => {
                self.handle_diagnostic(msg);
            }
            Message::BuildScriptExecuted(msg) => {
                self.report_compile_step(serde_json::to_string(&msg).ok());
            }
            Message::BuildFinished(msg) => {
                self.finish_compile(msg);
            }
            Message::TextLine(msg) => {
                let deserialized_message = serde_json::from_str::<TestType>(&msg);
                match deserialized_message {
                    // Message comes from running tests.
                    Ok(test_type) => self.handle_information_from_test(test_type),
                    // Message is a line from stdout.
                    Err(_) => self.log_message(MessageType::Log, msg, None),
                }
            }
            _ => (),
        }
    }

    fn report_compile_step(&mut self, msg: Option<String>) {
        self.state.compile_state.increase_compilation_step();
        self.report_task_progress(
            self.state.compile_state.task_id.clone(),
            msg,
            self.state.unit_graph_state.total_compilation_steps,
            self.state.compile_state.compilation_step,
            Some("compilation_steps".to_string()),
        );
    }

    fn handle_diagnostic(&mut self, msg: CompilerMessage) {
        // Diagnostics in Cargo are identified by root path, however in BSP
        // they are identified by the BuildTargetId.
        let abs_root_path = match self.root_path.absolutize() {
            Ok(path) => path.to_path_buf(),
            Err(e) => {
                warn!("Couldn't find absolute path for project's root path: {}", e);
                return;
            }
        };
        let build_target_id = match self.src_path_to_target_id.get(&msg.target.src_path) {
            Some(id) => id,
            None => {
                warn!(
                    "Target with path {} not found. Cannot publish diagnostic",
                    msg.target.src_path
                );
                return;
            }
        };
        let diagnostic_msg = map_cargo_diagnostic_to_bsp(
            &msg.message,
            self.params.origin_id(),
            build_target_id,
            AbsPath::assert(&abs_root_path),
        );
        match diagnostic_msg {
            DiagnosticMessage::Diagnostics(diagnostics) => {
                self.publish_diagnostic(diagnostics);
            }
            DiagnosticMessage::GlobalMessage(global_msg) => self.send_global_message(global_msg),
        }
    }

    fn publish_diagnostic(&mut self, diagnostics: Vec<PublishDiagnosticsParams>) {
        diagnostics.into_iter().for_each(|diagnostic| {
            // Count errors and warnings.
            diagnostic.diagnostics.iter().for_each(|d| {
                if let Some(severity) = d.severity {
                    match severity {
                        DiagnosticSeverity::ERROR => self.state.compile_state.errors += 1,
                        DiagnosticSeverity::WARNING => self.state.compile_state.warnings += 1,
                        _ => (),
                    }
                }
            });
            self.send_notification::<OnBuildPublishDiagnostics>(diagnostic);
        })
    }

    fn send_global_message(&self, global_msg: GlobalMessage) {
        let message_type = match global_msg.level {
            DiagnosticLevel::Ice | DiagnosticLevel::Error => MessageType::Error,
            DiagnosticLevel::Warning => MessageType::Warning,
            DiagnosticLevel::FailureNote | DiagnosticLevel::Note | DiagnosticLevel::Help => {
                MessageType::Info
            }
            _ => MessageType::Log,
        };
        self.send_notification::<OnBuildLogMessage>(LogMessageParams {
            message_type,
            task: Some(self.state.compile_state.task_id.clone()),
            origin_id: self.params.origin_id(),
            message: global_msg.message,
        });
    }

    fn finish_compile(&mut self, msg: BuildFinished) {
        self.build_targets.iter().for_each(|id| {
            // We can unwrap here, as for all iterated ids, the target state was created.
            let compile_target_state = self.state.compile_state.target_states.get(id).unwrap();
            let compile_report = TaskDataWithKind::CompileReport(CompileReportData {
                target: id.clone(),
                origin_id: self.params.origin_id(),
                errors: self.state.compile_state.errors,
                warnings: self.state.compile_state.warnings,
                time: Some(get_current_time() - compile_target_state.start_time),
                no_op: None,
            });
            self.report_task_finish(
                compile_target_state.task_id.clone(),
                StatusCode::Ok,
                None,
                Some(compile_report),
            );
        });
        self.report_task_finish(
            self.state.compile_state.task_id.clone(),
            StatusCode::Ok,
            Some("Finished compilation".to_string()),
            None,
        );
        // Start execution task if compile finished with success.
        if msg.success {
            self.start_execution_task()
        } else {
            self.state.task_state = TaskState::Compile
        }
    }

    fn start_execution_task(&self) {
        match &self.state.task_state {
            TaskState::Compile => (),
            TaskState::Run(run_state) => self.report_task_start(
                run_state.task_id.clone(),
                Some("Started target execution".to_string()),
                None,
            ),
            TaskState::Test(test_state) => self.report_task_start(
                test_state.task_id.clone(),
                Some("Started target testing".to_string()),
                None,
            ),
        }
    }

    fn handle_information_from_test(&mut self, test_type: TestType) {
        match test_type {
            TestType::Suite(event) => self.handle_test_suite(event),
            TestType::Test(event) => self.handle_single_test(event),
        }
    }

    fn handle_test_suite(&mut self, event: SuiteEvent) {
        if let TaskState::Test(test_state) = &mut self.state.task_state {
            let mut task_id = test_state.suite_task_id.clone();
            match event {
                SuiteEvent::Started(s) => {
                    let new_id = generate_random_id();
                    task_id.id = new_id.clone();
                    test_state.suite_task_id.id = new_id;
                    test_state.suite_task_progress.total = s.test_count as i64;
                    test_state.suite_task_progress.progress = 0;
                    // Because the targets are sorted, we know which one is currently tested.
                    test_state.current_build_target = self.build_targets.pop();
                    let target = match test_state.current_build_target.clone() {
                        Some(t) => t,
                        None => {
                            warn!("Test suite generated for unknown build target");
                            return;
                        }
                    };
                    self.report_task_start(
                        task_id,
                        None,
                        Some(TaskDataWithKind::TestTask(TestTaskData { target })),
                    );
                }
                SuiteEvent::Ok(result) | SuiteEvent::Failed(result) => {
                    test_state.suite_task_progress = SuiteTaskProgress::default();
                    self.report_suite_finished(task_id, result)
                }
            }
        }
    }

    fn report_suite_finished(&self, task_id: TaskId, result: SuiteResults) {
        if let TaskState::Test(test_state) = &self.state.task_state {
            let tested_target = match test_state.current_build_target.clone() {
                Some(t) => t,
                None => {
                    warn!("No target is currently tested");
                    return;
                }
            };
            self.report_task_finish(
                task_id,
                StatusCode::Ok,
                None,
                Some(result.to_test_report(tested_target)),
            )
        }
    }

    fn handle_single_test(&mut self, event: TestEvent) {
        if let TaskState::Test(test_state) = &mut self.state.task_state {
            match event {
                TestEvent::Started(started) => {
                    let test_task_id = generate_task_id(&test_state.suite_task_id);
                    test_state
                        .single_test_task_ids
                        .insert(started.name.clone(), test_task_id.clone());
                    self.report_task_start(
                        test_task_id,
                        None,
                        Some(TaskDataWithKind::TestStart(TestStartData {
                            display_name: started.name,
                            // TODO add location of build target
                            location: None,
                        })),
                    );
                }
                TestEvent::Ok(result) => self.finish_single_test(result, TestStatus::Passed),
                TestEvent::Failed(result) => self.finish_single_test(result, TestStatus::Failed),
                TestEvent::Ignored(result) => self.finish_single_test(result, TestStatus::Ignored),
                TestEvent::Timeout(result) => self.finish_single_test(result, TestStatus::Failed),
            }
        }
    }

    fn finish_single_test(&mut self, mut test_result: TestResult, status: TestStatus) {
        if let TaskState::Test(test_state) = &mut self.state.task_state {
            if let Some(id) = test_state.single_test_task_ids.remove(&test_result.name) {
                let test_task_id = test_state.suite_task_id.clone();
                let total = test_state.suite_task_progress.total;
                let progress = test_state.suite_task_progress.progress + 1;
                test_state.suite_task_progress.progress = progress;
                if let Some(message) = test_result.handle_test_stdout() {
                    self.log_message(MessageType::Log, message, Some(id.clone()));
                }
                self.report_task_finish(
                    id,
                    StatusCode::Ok,
                    None,
                    Some(TaskDataWithKind::TestFinish(
                        test_result.map_to_test_notification(status),
                    )),
                );
                self.report_task_progress(
                    test_task_id,
                    None,
                    Some(total),
                    Some(progress),
                    Some("tests".to_string()),
                );
            }
        }
    }
}
