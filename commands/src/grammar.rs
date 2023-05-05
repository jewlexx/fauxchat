use pest::Parser;

#[derive(Parser)]
#[grammar = "../grammar.pest"]
pub struct CommandsParser;

impl CommandsParser {
    pub fn commands(input: &str) -> Result<Vec<super::Command>, pest::error::Error<Rule>> {
        let mut commands = vec![];

        let ast = input
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| CommandsParser::parse(Rule::line, line))
            .collect::<Result<Vec<_>, _>>()?;

        for line in ast {
            // Should only contain one line
            assert_eq!(line.len(), 1);
            let line = {
                let mut line = line.clone();
                line.next().unwrap()
            };

            let mut inner_pairs = line.into_inner();

            // Should only contain one expression, and one endline
            assert_eq!(inner_pairs.len(), 2);
            let command = {
                let command = inner_pairs.next().unwrap();
                if command.as_rule() != Rule::command_single {
                    continue;
                }

                command
            };

            let command_parts = command.into_inner();

            let mut command_name = "";
            let mut args: Vec<String> = vec![];

            for part in command_parts {
                match part.as_rule() {
                    Rule::command_action => {
                        // Should only ever be assigned once
                        assert!(command_name.is_empty());

                        command_name = part.as_str();
                    }
                    Rule::command_argument => {
                        for arg in part.into_inner() {
                            args.push(arg.as_str().to_string());
                        }
                    }
                    _ => continue,
                }

                // dbg!(part);

                // match rule {
                // }
            }

            let command = match command_name {
                "send" => {
                    let message = args[0].to_string();
                    let times = {
                        let times = &args[1];
                        if times.is_empty() {
                            0_u64
                        } else {
                            times.parse().unwrap()
                        }
                    };

                    super::Command::Send(message, times)
                }
                "sleep" => {
                    let ms = args[0].parse().unwrap();
                    super::Command::Sleep(ms)
                }
                _ => panic!("Unknown command: {command_name}"),
            };

            commands.push(command);
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    const MESSAGES: &str = include_str!("../../messages.txt");

    #[test]
    fn test_parse() {
        let commands = super::CommandsParser::commands(MESSAGES).unwrap();

        dbg!(commands);
    }
}
