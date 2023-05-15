use std::{fmt, panic};

use bsp_server::{ErrorCode, ExtractError, Notification, Request, RequestId, Response};
use log::warn;
use serde::{de::DeserializeOwned, Serialize};

use bsp_types;

use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::request_handle::RequestHandle;
use crate::server::global_state::{GlobalState, GlobalStateSnapshot};
use crate::server::{from_json, LspError, Result};

pub(crate) struct RequestDispatcher<'a> {
    pub(crate) req: Option<Request>,
    pub(crate) global_state: &'a mut GlobalState,
}

impl<'a> RequestDispatcher<'a> {
    /// Dispatches the request onto the current thread, given full access to
    /// mutable global state.
    pub(crate) fn on_sync_mut<R>(
        &mut self,
        f: fn(&mut GlobalState, R::Params) -> Result<R::Result>,
    ) -> &mut Self
    where
        R: bsp_types::requests::Request,
        R::Params: DeserializeOwned + panic::UnwindSafe + fmt::Debug,
        R::Result: Serialize,
    {
        let (req, params, _) = match self.parse::<R>() {
            Some(it) => it,
            None => return self,
        };
        let result = { f(self.global_state, params) };
        if let Ok(response) = result_to_response::<R>(req.id, result) {
            self.global_state.respond(response);
        }

        self
    }

    /// Dispatches the request onto the current thread.
    pub(crate) fn on_sync<R>(
        &mut self,
        f: fn(GlobalStateSnapshot, R::Params) -> Result<R::Result>,
    ) -> &mut Self
    where
        R: bsp_types::requests::Request,
        R::Params: DeserializeOwned + panic::UnwindSafe + fmt::Debug,
        R::Result: Serialize,
    {
        let (req, params, _) = match self.parse::<R>() {
            Some(it) => it,
            None => return self,
        };
        let global_state_snapshot = self.global_state.snapshot();

        let result = { f(global_state_snapshot, params) };

        if let Ok(response) = result_to_response::<R>(req.id, result) {
            self.global_state.respond(response);
        }

        self
    }

    pub(crate) fn on_cargo_run<R>(&mut self) -> &mut Self
    where
        R: bsp_types::requests::Request + 'static,
        R::Params: CreateCommand + Send + fmt::Debug,
        R::Result: Serialize + CargoResult,
    {
        let (req, params, _) = match self.parse::<R>() {
            Some(it) => it,
            None => return self,
        };
        let sender_to_main = self.global_state.handlers_sender.clone();
        let request_handle = RequestHandle::spawn::<R>(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            req.id.clone(),
            params,
            self.global_state.config.root_path(),
        );
        self.global_state.handlers.insert(req.id, request_handle);

        self
    }

    pub(crate) fn finish(&mut self) {
        if let Some(req) = self.req.take() {
            warn!("unknown request: {:?}", req);
            let response = Response::new_err(
                req.id,
                ErrorCode::MethodNotFound as i32,
                "unknown request".to_string(),
            );
            self.global_state.respond(response);
        }
    }

    fn parse<R>(&mut self) -> Option<(Request, R::Params, String)>
    where
        R: bsp_types::requests::Request,
        R::Params: DeserializeOwned + fmt::Debug,
    {
        let req = match &self.req {
            Some(req) if req.method == R::METHOD => self.req.take()?,
            _ => return None,
        };

        let res = from_json(R::METHOD, &req.params);
        match res {
            Ok(params) => {
                let panic_context = format!("\nrequest: {} {:#?}", R::METHOD, params);
                Some((req, params, panic_context))
            }
            Err(err) => {
                let response =
                    Response::new_err(req.id, ErrorCode::InvalidParams as i32, err.to_string());
                self.global_state.respond(response);
                None
            }
        }
    }
}

fn result_to_response<R>(id: RequestId, result: Result<R::Result>) -> Result<Response>
where
    R: bsp_types::requests::Request,
    R::Params: DeserializeOwned,
    R::Result: Serialize,
{
    let res = match result {
        Ok(resp) => bsp_server::Response::new_ok(id, &resp),
        Err(e) => match e.downcast::<LspError>() {
            Ok(lsp_error) => Response::new_err(id, lsp_error.code, lsp_error.message),
            Err(e) => Response::new_err(id, ErrorCode::InternalError as i32, e.to_string()),
        },
    };
    Ok(res)
}

pub(crate) struct NotificationDispatcher<'a> {
    pub(crate) not: Option<Notification>,
    pub(crate) global_state: &'a mut GlobalState,
}

impl<'a> NotificationDispatcher<'a> {
    pub(crate) fn on<N>(
        &mut self,
        f: fn(&mut GlobalState, N::Params) -> Result<()>,
    ) -> Result<&mut Self>
    where
        N: bsp_types::notifications::Notification,
        N::Params: DeserializeOwned + Send,
    {
        let not = match self.not.take() {
            Some(it) => it,
            None => return Ok(self),
        };
        let params = match not.extract::<N::Params>(N::METHOD) {
            Ok(it) => it,
            Err(ExtractError::JsonError { method, error }) => {
                panic!("Invalid request\nMethod: {method}\n error: {error}",)
            }
            Err(ExtractError::MethodMismatch(not)) => {
                self.not = Some(not);
                return Ok(self);
            }
        };
        f(self.global_state, params)?;
        Ok(self)
    }

    pub(crate) fn finish(&mut self) {
        if let Some(not) = &self.not {
            if !not.method.starts_with("$/") {
                warn!("unhandled notification: {:?}", not);
            }
        }
    }
}