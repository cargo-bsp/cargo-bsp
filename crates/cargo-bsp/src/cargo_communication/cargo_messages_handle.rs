use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::{BuildFinished, CompilerMessage, Message};
use log::warn;
use lsp_types::DiagnosticSeverity;
use path_absolutize::*;
use paths::AbsPath;

use bsp_types::notifications::{
    CompileReportData, LogMessage, LogMessageParams, MessageType, PublishDiagnostics,
    PublishDiagnosticsParams, TaskDataWithKind, TaskId, TestStartData, TestStatus, TestTaskData,
};
use bsp_types::requests::Request;
use bsp_types::{BuildTargetIdentifier, StatusCode};

use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::CargoMessage;
use crate::cargo_communication::cargo_types::publish_diagnostics::{
    map_cargo_diagnostic_to_bsp, DiagnosticMessage, GlobalMessage,
};
use crate::cargo_communication::cargo_types::test::{
    SuiteEvent, SuiteResults, TestEvent, TestResult, TestType,
};
use crate::cargo_communication::request_actor::{CargoHandler, RequestActor};
use crate::cargo_communication::request_actor_state::TaskState;
use crate::cargo_communication::utils::{generate_random_id, generate_task_id, get_current_time};

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand,
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

    fn report_compile_step(&self, msg: Option<String>) {
        self.report_task_progress(
            self.state.compile_state.task_id.clone(),
            msg,
            None,
            None,
            None,
        );
    }

    fn handle_diagnostic(&mut self, msg: CompilerMessage) {
        let abs_root_path = match self.root_path.absolutize() {
            Ok(path) => path.to_path_buf(),
            Err(e) => {
                warn!("Couldn't find absolute path for project's root path: {}", e);
                return;
            }
        };
        let diagnostic_msg = map_cargo_diagnostic_to_bsp(
            &msg.message,
            self.params.origin_id(),
            // TODO change to actual BuildTargetIdentifier
            &BuildTargetIdentifier::default(),
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
            self.send_notification::<PublishDiagnostics>(diagnostic);
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
        self.send_notification::<LogMessage>(LogMessageParams {
            message_type,
            task: Some(self.state.compile_state.task_id.clone()),
            origin_id: self.params.origin_id(),
            message: global_msg.message,
        });
    }

    fn finish_compile(&mut self, msg: BuildFinished) {
        let compile_report = TaskDataWithKind::CompileReport(CompileReportData {
            // TODO change to actual BuildTargetIdentifier
            target: BuildTargetIdentifier::default(),
            origin_id: self.params.origin_id(),
            errors: self.state.compile_state.errors,
            warnings: self.state.compile_state.warnings,
            time: Some((get_current_time() - self.state.compile_state.start_time) as i32),
            no_op: None,
        });
        self.report_task_finish(
            self.state.compile_state.task_id.clone(),
            StatusCode::Ok,
            None,
            Some(compile_report),
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
                    self.report_task_start(
                        task_id,
                        None,
                        // TODO change target to actual BuildTargetIdentifier
                        Some(TaskDataWithKind::TestTask(TestTaskData::default())),
                    );
                }
                SuiteEvent::Ok(result) | SuiteEvent::Failed(result) => {
                    self.report_suite_finished(task_id, result)
                }
            }
        }
    }

    fn report_suite_finished(&self, task_id: TaskId, result: SuiteResults) {
        self.report_task_finish(task_id, StatusCode::Ok, None, Some(result.to_test_report()))
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
                        // TODO to be deleted, when client allows empty message
                        Some("Test started".to_string()),
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
                    // TODO to be deleted, when client allows empty message
                    Some("Test finished".to_string()),
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
