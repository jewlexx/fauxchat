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

        for pair in ast {
            for p in pair {
                dbg!(p);
            }
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
