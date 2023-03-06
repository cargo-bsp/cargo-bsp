// copy from rust-analyzer

use std::{fmt, panic};

use serde::{de::DeserializeOwned, Serialize};

use crate::{bsp_types, communication};
use crate::communication::ExtractError;
use crate::logger::log;
use crate::server::{from_json, LspError};
use crate::server::global_state::GlobalState;
use crate::server::request_actor::RequestHandle;
use crate::server::Result;

pub(crate) struct RequestDispatcher<'a> {
    pub(crate) req: Option<communication::Request>,
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

    pub(crate) fn on_running_cargo<R>(
        &mut self,
        f: fn(&mut GlobalState, R::Params, &communication::RequestId) -> Result<RequestHandle>,
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
        let result = { f(self.global_state, params, &req.id) };
        if let Ok(handle) = result {
            self.global_state.handlers.insert(req.id, handle);
        }

        self
    }

    pub(crate) fn finish(&mut self) {
        if let Some(req) = self.req.take() {
            log(&format!("unknown request: {:?}", req));
            let response = communication::Response::new_err(
                req.id,
                communication::ErrorCode::MethodNotFound as i32,
                "unknown request".to_string(),
            );
            self.global_state.respond(response);
        }
    }

    fn parse<R>(&mut self) -> Option<(communication::Request, R::Params, String)>
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
                let response = communication::Response::new_err(
                    req.id,
                    communication::ErrorCode::InvalidParams as i32,
                    err.to_string(),
                );
                self.global_state.respond(response);
                None
            }
        }
    }
}

fn result_to_response<R>(
    id: communication::RequestId,
    result: Result<R::Result>,
) -> Result<communication::Response>
    where
        R: bsp_types::requests::Request,
        R::Params: DeserializeOwned,
        R::Result: Serialize,
{
    let res = match result {
        Ok(resp) => communication::Response::new_ok(id, &resp),
        Err(e) => match e.downcast::<LspError>() {
            Ok(lsp_error) => {
                communication::Response::new_err(id, lsp_error.code, lsp_error.message)
            }
            Err(e) => communication::Response::new_err(
                id,
                communication::ErrorCode::InternalError as i32,
                e.to_string(),
            ),
        },
    };
    Ok(res)
}

pub(crate) struct NotificationDispatcher<'a> {
    pub(crate) not: Option<communication::Notification>,
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
                panic!("Invalid request\nMethod: {method}\n error: {error}", )
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
                log(&format!("unhandled notification: {:?}", not));
            }
        }
    }
}
