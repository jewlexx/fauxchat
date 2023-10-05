use std::borrow::Cow;

use deno_core::{error::AnyError, extension, op2, Op, OpDecl};
use deno_runtime::deno_core;

extension!(chat_commands, ops = [op_send]);

#[op2(async)]
pub async fn op_send(
    #[string] message: String,
    #[bigint] count: usize,
    #[bigint] delay: u64,
    #[string] username: String,
) -> Result<(), AnyError> {
    unimplemented!()
}
