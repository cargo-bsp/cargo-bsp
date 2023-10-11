pub fn bsp_request_id_to_lsp_request_id(
    request_id: bsp_types::cancel::RequestId,
) -> bsp_server::RequestId {
    match request_id {
        bsp_types::cancel::RequestId::I32(id) => id.into(),
        bsp_types::cancel::RequestId::String(id) => id.into(),
    }
}
