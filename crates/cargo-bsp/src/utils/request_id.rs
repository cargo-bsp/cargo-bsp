pub fn bsp_request_id_to_lsp_request_id(
    request_id: bsp_types::extensions::RequestId,
) -> bsp_server::RequestId {
    match request_id {
        bsp_types::extensions::RequestId::I32(id) => id.into(),
        bsp_types::extensions::RequestId::String(id) => id.into(),
    }
}
