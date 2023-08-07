use pest::Parser;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    // The string provided is the error message from pest
    #[error("{0}")]
    ParsingError(String),
    #[error("The command provided was invalid. Found {0}")]
    InvalidCommand(String),
    #[error("The command given was a comment")]
    Comment,
}

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Copy, Clone)]
pub struct CommandInfo {
    /// A standard command name, in lowercase
    pub name: &'static str,
    /// The number of arguments the command takes
    pub arg_count: usize,
}

impl CommandInfo {
    pub fn from_name(cmd_name: &str) -> Result<CommandInfo> {
        match cmd_name.to_lowercase().as_str() {
            "send" => Ok(CommandInfo {
                name: "send",
                arg_count: 4,
            }),
            "sleep" => Ok(CommandInfo {
                name: "sleep",
                arg_count: 1,
            }),
            _ => Err(ParseError::InvalidCommand(cmd_name.to_string())),
        }
    }
}

#[derive(Parser)]
#[grammar = "../grammar.pest"]
pub struct CommandsParser;

impl CommandsParser {
    /// Parses a single command.
    ///
    /// # Panics
    /// - Will panic if the input is invalid.
    pub fn parse_parts(input: &str) -> Result<Vec<&str>> {
        if CommandsParser::parse(Rule::comment_single, input).is_ok() {
            return Err(ParseError::Comment);
        }

        let mut ast = CommandsParser::parse(Rule::command_single, input)
            .map_err(|e| ParseError::ParsingError(e.to_string()))?;

        // Should only be a single command
        assert_eq!(ast.len(), 1);

        let cmd = ast.next().unwrap();

        let mut parts = cmd.into_inner();

        let cmd_name = {
            let part = parts.next().unwrap();
            assert_eq!(part.as_rule(), Rule::command_action);

            part.as_str()
        };

        let cmd_info = super::CommandInfo::from_name(cmd_name)?;

        let mut args = Vec::with_capacity(cmd_info.arg_count);

        for _ in 0..cmd_info.arg_count {
            let Some(part) = parts.next() else { break; };
            assert_eq!(part.as_rule(), Rule::command_argument);
            args.push(part.as_str());
        }

        let mut with_name = vec![cmd_info.name];
        with_name.extend(args.iter());

        Ok(with_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{amount, Command};

    #[test]
    fn test_parse() {
        // Send "Message Here" 10 times with 10 milliseconds in between each
        let command = {
            let parts = CommandsParser::parse_parts("send(\"Message Here\", 10, 10)").unwrap();
            Command::from_parts(&parts).unwrap()
        };
        let act = Command::Send {
            message: String::from("Message Here"),
            username: String::from("random"),
            count: 10,
            delay: amount::Amount::Single(10),
        };

        assert_eq!(command, act);

        // const MESSAGES: &str = include_str!("../../../messages.txt");

        // let commands = CommandsParser::commands(MESSAGES).unwrap();
        // let mut commands_iter = commands.iter();

        // let command = commands_iter.next().unwrap();

        // assert_eq!(command, &Command::Send("Hey!".to_string(), 10));

        // let command = commands_iter.next().unwrap();

        // assert_eq!(command, &Command::Send("Hello world!".to_string(), 1));

        // let command = commands_iter.next().unwrap();

        // assert_eq!(command, &Command::Sleep(1000));

        // // Ensure that we have exhausted the iterator
        // assert!(commands_iter.next().is_none());
    }

    #[test]
    fn test_parse_comment() {
        let comment_error =
            CommandsParser::parse_parts("// This is a comment").expect_err("error value");

        assert_eq!(comment_error, ParseError::Comment);
    }
}
