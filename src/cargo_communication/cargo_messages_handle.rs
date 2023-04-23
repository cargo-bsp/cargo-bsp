use crate::bsp_types::mappings::test::{SuiteEvent, TestEvent, TestResult, TestType};
use crate::bsp_types::mappings::to_publish_diagnostics::{
    map_cargo_diagnostic_to_bsp, DiagnosticMessage, GlobalMessage,
};
use crate::bsp_types::notifications::{
    get_event_time, CompileReportData, LogMessage, LogMessageParams, MessageType,
    PublishDiagnostics, PublishDiagnosticsParams, TaskDataWithKind, TaskId, TestStartData,
    TestStatus, TestTaskData,
};
use crate::bsp_types::requests::{CreateCommand, CreateResult, Request};
use crate::bsp_types::{BuildTargetIdentifier, StatusCode};
use crate::cargo_communication::request_actor::{
    CargoHandleTrait, CargoMessage, ExecutionState, RequestActor,
};
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::{BuildFinished, CompilerMessage, Message};
use lsp_types::DiagnosticSeverity;
use paths::AbsPath;

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand + CreateResult<R::Result>,
    C: CargoHandleTrait<CargoMessage>,
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
            self.state.compile_state.compile_task_id.clone(),
            msg,
            None,
            None,
            None,
        );
    }

    fn handle_diagnostic(&mut self, msg: CompilerMessage) {
        let diagnostic_msg = map_cargo_diagnostic_to_bsp(
            &msg.message,
            self.params.origin_id(),
            // TODO change to actual BuildTargetIdentifier
            &BuildTargetIdentifier {
                uri: "".to_string(),
            },
            AbsPath::assert(&self.root_path),
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
                        DiagnosticSeverity::ERROR => self.state.compile_state.compile_errors += 1,
                        DiagnosticSeverity::WARNING => {
                            self.state.compile_state.compile_warnings += 1
                        }
                        _ => (),
                    }
                }
            });
            self.send_notification::<PublishDiagnostics>(diagnostic);
        })
    }

    fn send_global_message(&self, global_msg: GlobalMessage) {
        let message_type = match global_msg.level {
            DiagnosticLevel::Ice => MessageType::Error,
            DiagnosticLevel::Error => MessageType::Error,
            DiagnosticLevel::Warning => MessageType::Warning,
            DiagnosticLevel::FailureNote => MessageType::Info,
            DiagnosticLevel::Note => MessageType::Info,
            DiagnosticLevel::Help => MessageType::Info,
            _ => MessageType::Log,
        };
        self.send_notification::<LogMessage>(LogMessageParams {
            message_type,
            task: Some(self.state.compile_state.compile_task_id.clone()),
            origin_id: self.params.origin_id(),
            message: global_msg.message,
        });
    }

    fn finish_compile(&self, msg: BuildFinished) {
        let status_code = if msg.success {
            StatusCode::Ok
        } else {
            StatusCode::Error
        };
        let compile_report = TaskDataWithKind::CompileReport(CompileReportData {
            // TODO change to actual BuildTargetIdentifier
            target: Default::default(),
            origin_id: self.params.origin_id(),
            errors: self.state.compile_state.compile_errors,
            warnings: self.state.compile_state.compile_warnings,
            time: Some(
                (get_event_time().unwrap() - self.state.compile_state.compile_start_time) as i32,
            ),
            no_op: None,
        });
        self.report_task_finish(
            self.state.compile_state.compile_task_id.clone(),
            status_code,
            None,
            Some(compile_report),
        );
        self.start_execution_task();
    }

    fn start_execution_task(&self) {
        match &self.state.execution_state {
            ExecutionState::Compile => (),
            ExecutionState::Run(run_state) => self.report_task_start(
                run_state.run_task_id.clone(),
                Some("Started target execution".to_string()),
                None,
            ),
            ExecutionState::Test(test_state) => self.report_task_start(
                test_state.test_task_id.clone(),
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
        if let ExecutionState::Test(test_state) = &mut self.state.execution_state {
            let mut task_id = test_state.suite_test_task_id.clone();
            match event {
                SuiteEvent::Started(s) => {
                    let new_task_id = TaskId::generate_random_id();
                    task_id.id = new_task_id.clone();
                    test_state.suite_test_task_id.id = new_task_id;
                    test_state.suite_task_progress.total = s.test_count as i64;
                    self.report_task_start(
                        task_id,
                        None,
                        // TODO change target to actual BuildTargetIdentifier
                        Some(TaskDataWithKind::TestTask(TestTaskData {
                            target: Default::default(),
                        })),
                    );
                }
                SuiteEvent::Ok(result) => self.report_task_finish(
                    task_id,
                    StatusCode::Ok,
                    None,
                    Some(result.to_test_report()),
                ),
                SuiteEvent::Failed(result) => self.report_task_finish(
                    task_id,
                    StatusCode::Error,
                    None,
                    Some(result.to_test_report()),
                ),
            }
        }
    }

    fn handle_single_test(&mut self, event: TestEvent) {
        if let ExecutionState::Test(test_state) = &mut self.state.execution_state {
            match event {
                TestEvent::Started(started) => {
                    let test_task_id = TaskId {
                        id: TaskId::generate_random_id(),
                        parents: vec![test_state.suite_test_task_id.id.clone()],
                    };
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
        if let ExecutionState::Test(test_state) = &mut self.state.execution_state {
            if let Some(id) = test_state.single_test_task_ids.remove(&test_result.name) {
                let test_task_id = test_state.suite_test_task_id.clone();
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
