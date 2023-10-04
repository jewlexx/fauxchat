use deno_core::{error::AnyError, op, Op, OpDecl};

pub fn declarations() -> Vec<OpDecl> {
    vec![op_send::DECL]
}

#[op]
pub async fn op_send(
    message: String,
    count: usize,
    delay: u64,
    username: String,
) -> Result<(), AnyError> {
    todo!()
}
