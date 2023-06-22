use pest::Parser;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("{0}")]
    ParsingError(#[from] pest::error::Error<Rule>),

    #[error("The command provided was invalid. Found {0}")]
    InvalidCommand(String),
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
                arg_count: 3,
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
        let mut ast = CommandsParser::parse(Rule::command_single, input)?;

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
            let part = parts.next().unwrap();
            assert_eq!(part.as_rule(), Rule::command_argument);
            args.push(part.as_str());
        }

        let mut with_name = vec![cmd_info.name];
        with_name.extend(args.iter());

        // let parsed_command = super::Command::from_parts(&with_name)?;

        Ok(with_name)
    }

    // pub fn commands(input: &str) -> anyhow::Result<Vec<super::Command>> {
    //     let mut commands = vec![];

    //     let ast = input
    //         .lines()
    //         .filter(|line| !line.is_empty())
    //         .map(|line| CommandsParser::parse(Rule::line, line))
    //         .collect::<Result<Vec<_>, _>>()?;

    //     for line in ast {
    //         // Should only contain one line
    //         assert_eq!(line.len(), 1);
    //         let line = {
    //             let mut line = line.clone();
    //             line.next().unwrap()
    //         };

    //         let mut inner_pairs = line.into_inner();

    //         // Should only contain one expression, and one endline
    //         assert_eq!(inner_pairs.len(), 2);
    //         let command = {
    //             let command = inner_pairs.next().unwrap();
    //             if command.as_rule() != Rule::command_single {
    //                 continue;
    //             }

    //             command
    //         };

    //         let command_parts = command.into_inner();

    //         let mut command_name = "";
    //         let mut args: Vec<String> = vec![];

    //         for part in command_parts {
    //             match part.as_rule() {
    //                 Rule::command_action => {
    //                     // Should only ever be assigned once
    //                     assert!(command_name.is_empty());

    //                     command_name = part.as_str();
    //                 }
    //                 Rule::command_argument => {
    //                     for arg in part.into_inner() {
    //                         args.push(arg.as_str().to_string());
    //                     }
    //                 }
    //                 _ => continue,
    //             }
    //         }

    //         // TODO: Use litrs
    //         let command = match command_name {
    //             "send" => {
    //                 let message = {
    //                     let arg = args.remove(0);
    //                     let lit = litrs::StringLit::parse(arg)?;
    //                     let lit_value = lit.value();

    //                     lit_value.to_string()
    //                 };
    //                 let times = {
    //                     if args.is_empty() {
    //                         1_u64
    //                     } else {
    //                         args.remove(0).parse().unwrap()
    //                     }
    //                 };

    //                 super::Command::Send(message, times)
    //             }
    //             "sleep" => {
    //                 let ms = args[0].parse().unwrap();
    //                 super::Command::Sleep(ms)
    //             }
    //             _ => panic!("Unknown command: {command_name}"),
    //         };

    //         commands.push(command);
    //     }

    //     Ok(commands)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    #[test]
    fn test_parse() {
        // Send "Message Here" 10 times with 10 milliseconds in between each
        let command = {
            let parts = CommandsParser::parse_parts("send(\"Message Here\", 10, 10)").unwrap();
            Command::from_parts(&parts).unwrap()
        };
        let act = Command::Send {
            message: String::from("Message Here"),
            count: 10,
            delay: 10,
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
}
