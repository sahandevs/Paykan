use std::mem::swap;

#[derive(Debug, PartialEq)]
pub enum ParameterType {
    SingleQuote,
    DoubleQuote,
    Simple,
}
#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub parameters: Vec<(ParameterType, String)>,
    pub block: Option<Block>,
}

#[derive(Debug)]
pub struct Block {
    pub directives: Vec<Directive>,
}

fn parse_block(input: &str) -> (Block, &str) {
    let mut block = Block {
        directives: Vec::new(),
    };
    let mut rest = input;
    let mut last_index;
    'outer: loop {
        'inner: for (i, char) in rest.chars().enumerate() {
            last_index = i;
            match char {
                '}' => break 'outer,
                ' ' | '\t' | '\n' => continue,
                _ => {}
            };
            let result = parse_directive(rest);
            block.directives.push(result.0);
            rest = result.1;
            if rest.trim() == "" {
                break 'outer;
            }
            break 'inner;
        }
    }
    let rest = if rest.trim() == "" {
        ""
    } else {
        &rest[last_index..]
    };
    (block, rest)
}

pub fn parse_directive(input: &str) -> (Directive, &str) {
    let mut name = String::new();
    let mut parameters = Vec::new();

    let mut parameter_type = ParameterType::Simple;
    let mut parameter_value = String::new();
    let mut is_parameter_ended = true;
    let mut is_name = true;
    let mut last_index = 0;
    let mut block = None;
    let mut rest = None;
    for (i, char) in input.chars().enumerate() {
        last_index = i;
        if is_name {
            if name.is_empty() {
                if char.is_numeric() {
                    panic!("Parser error: expected character found number {}", char);
                } else if char == '\n' || char == '\r' || char == ' ' {
                    continue;
                }
            } else if char == ' ' {
                is_name = false;
                continue;
            } else if char == ';' {
                break;
            }
            name.push(char);
        } else {
            if char == ';' {
                if let ParameterType::Simple = parameter_type {
                    if parameter_value.is_empty() {
                        break;
                    }
                }
                let param = (parameter_type, parameter_value);
                parameters.push(param);
                break;
            }
            if is_parameter_ended {
                match char {
                    ' ' => continue,
                    '{' => {
                        let result = parse_block(&input[last_index + 1..]);
                        block = Some(result.0);
                        rest = Some(result.1);
                        break;
                    }
                    _ => {}
                }
            }
            if parameter_value.is_empty() && is_parameter_ended {
                parameter_type = match char {
                    '\'' => ParameterType::SingleQuote,
                    '"' => ParameterType::DoubleQuote,
                    _ => {
                        parameter_value.push(char);
                        ParameterType::Simple
                    }
                };
                is_parameter_ended = false;
            } else {
                match (&parameter_type, char) {
                    (ParameterType::SingleQuote, '\'')
                    | (ParameterType::DoubleQuote, '"')
                    | (ParameterType::Simple, ' ' | '\t' | ';') => {
                        is_parameter_ended = true;
                        let mut value = String::new();
                        let mut par_type = ParameterType::Simple;
                        swap(&mut value, &mut parameter_value);
                        swap(&mut parameter_type, &mut par_type);
                        let param = (par_type, value);
                        parameters.push(param);
                        continue;
                    }
                    _ => {}
                };
                parameter_value.push(char);
            }
        }
    }
    (
        Directive {
            block,
            name,
            parameters,
        },
        rest.map(|x| &x[1..])
            .unwrap_or_else(|| &input[last_index + 1..]),
    )
}

pub fn parse(input: &str) -> Block {
    parse_block(input).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let result = &parse(
            r#"
directive1 'par1' par3 "'a" {
    hello 'test';
    hi a;
}
dir2;
        "#,
        );
        assert_eq!(result.directives.len(), 2);
        let directive1 = result.directives.get(0).unwrap();
        assert_eq!(directive1.parameters.len(), 3);
        assert_eq!(directive1.name, "directive1");
        let params = &directive1.parameters;

        assert_eq!(params.get(0).unwrap().0, ParameterType::SingleQuote);
        assert_eq!(params.get(1).unwrap().0, ParameterType::Simple);
        assert_eq!(params.get(2).unwrap().0, ParameterType::DoubleQuote);
        assert_eq!(params.get(0).unwrap().1, "par1");
        assert_eq!(params.get(1).unwrap().1, "par3");
        assert_eq!(params.get(2).unwrap().1, "'a");

        let block = directive1.block.as_ref().unwrap();
        assert_eq!(block.directives.len(), 2);
        assert_eq!(block.directives.get(0).unwrap().name, "hello");
        assert_eq!(block.directives.get(1).unwrap().name, "hi");
        assert_eq!(block.directives.get(0).unwrap().parameters.len(), 1);
        assert_eq!(block.directives.get(1).unwrap().parameters.len(), 1);
        assert_eq!(
            block
                .directives
                .get(0)
                .unwrap()
                .parameters
                .get(0)
                .unwrap()
                .0,
            ParameterType::SingleQuote
        );
        assert_eq!(
            block
                .directives
                .get(0)
                .unwrap()
                .parameters
                .get(0)
                .unwrap()
                .1,
            "test"
        );

        let directive2 = result.directives.get(1).unwrap();
        assert_eq!(directive2.name, "dir2");
        assert_eq!(directive2.parameters.len(), 0);
        assert!(directive2.block.is_none());
    }
}
