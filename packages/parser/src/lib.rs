use std::mem::swap;

#[derive(Debug)]
pub enum ParameterType {
    SingleQuote,
    DoubleQuote,
    Simple,
}
#[derive(Debug)]
pub struct Directive {
    name: String,
    parameters: Vec<(ParameterType, String)>,
    block: Option<Block>,
}

#[derive(Debug)]
struct Block {
    directives: Vec<Directive>,
}

fn parse_block(input: &str) -> Block {
    todo!()
}

pub fn parse_directive(input: &str) -> (Directive, &str) {
    let mut name = String::new();
    let mut parameters = Vec::new();

    let mut parameter_type = ParameterType::Simple;
    let mut parameter_value = String::new();
    let mut is_parameter_ended = true;
    let mut is_name = true;
    let mut last_index = 0;
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
            }
            name.push(char);
        } else {
            if is_parameter_ended {
                match char {
                    ' ' => continue,
                    '{' => {
                        // parse block
                        break;
                    },
                    ';' => {
                        break;
                    },
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
                    (ParameterType::SingleQuote, '\'') |
                    (ParameterType::DoubleQuote, '"') |
                    (ParameterType::Simple, ' ') => {
                        is_parameter_ended = true;
                        let mut value = String::new();
                        let mut par_type = ParameterType::Simple;
                        swap(&mut value, &mut parameter_value);
                        swap(&mut parameter_type, &mut par_type);
                        let param = (par_type, value);
                        parameters.push(param);
                        continue;
                    },
                    _ => {}
                };
                parameter_value.push(char);
            }
        }
    }
    (
        Directive {
            block: None,
            name,
            parameters,
        },
        &input[last_index+1..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let result = parse_directive(
            r#"
directive1 'par1' par3 "'a";
dir2;
        "#,
        );
        panic!("{:?}", result);
    }
}
