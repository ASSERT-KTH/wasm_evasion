//! Parsing of Souper IR to Lang

use std::{iter::Peekable, collections::HashMap};

use egg::RecExpr;
use wasm_mutate::mutators::peephole::eggsy::lang::Lang;

#[derive(Debug)]
struct SouperIRTree<T> {
    nodes: Vec<T>,
    operands: Vec<Vec<u32>>,
    parents: Vec<i32>,
    root: u32
}

fn get_number<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> u64 {
    let mut number = c.to_string().parse::<u64>().expect("The caller should have passed a digit.");
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<u64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    number
}

fn get_fixed_token<T: Iterator<Item = char>>(iter: &mut Peekable<T>, token: &mut Peekable<T>) -> bool {
    
    while let Some(c2) = token.next() {
        let c1 = iter.peek();
        match c1 {
            Some(&c1) => {

                if c1 != c2 {
                    return false
                }
            }
            None => {
                return false
            }
        }

        iter.next();
    }

    return true;
}

#[derive(Debug, Clone)]
enum Tokens {
    Assign,
    Integer(i32),
    Comma,
    EOL,
    Width(u64),
    Var(u64),
    Operator(String)
}

#[derive(Debug, Clone)]
enum MiddleNode {
    LangToken(Lang),
    LookupValue(String)
}


fn parses_operands(tokens: Vec<Tokens>, head: usize) -> anyhow::Result<(Vec<MiddleNode>, usize)> {
    
    let mut current = &tokens[head];
    let mut result: Vec<MiddleNode> = vec![];
    let mut position = head;
    match current {
        Tokens::Integer(val) => {
            match &tokens[head + 1] {
                Tokens::Width(w) => {
                    let integer = match w {
                        1 => {
                            // boolean
                            MiddleNode::LangToken(Lang::I32(*val))
                        }
                        32 => {
                            MiddleNode::LangToken(Lang::I32(*val))
                        }
                        _ => {
                            anyhow::bail!("Invalid size {}", w)
                        }
                    };
                    result.push(integer);
                    position += 2;
                    current = &tokens[position];
                },
                _ => anyhow::bail!("Missing width")
            }
        },
        Tokens::Var(id) => {
            result.push(MiddleNode::LookupValue(format!("%{}", id)));
            position += 1;
            current = &tokens[position];
        }
        _ => {
           anyhow::bail!("Invalid token {:?}", current);
        }
    };
    
    if let Tokens::Comma = current {
        // There are remaining operands
        let (remaining, newpos) = parses_operands(tokens.clone(), position + 1)?;
        result = [result, remaining].concat();
        position = newpos;
    }

    Ok((result, position))
}

fn parses_operation(tokens: Vec<Tokens>, head: usize, buffer: &mut HashMap<String, String>) -> anyhow::Result<(Option<(String, Vec<MiddleNode>)> /* If it is a variable definition then, there is no point in returning the node, register in the global map instead */, usize)> {
    let current = &tokens[head];

    match current {
        Tokens::Operator(id) => {
            match id.as_str() {
                "var" => {
                    // It is a variable declaration, add it to the global map
                    // and return if the next is EOL
                    if let Tokens::EOL = tokens[head + 1] {
                        return Ok((None, head + 2))
                    }

                }
                _ => {
                    // The remaining ones are probably operand having type
                    let (operands, pos) = parses_operands(tokens.clone(), head + 1)?;
                    if let Tokens::EOL = tokens[pos] {
                        return Ok((Some((
                            id.clone(),
                            operands
                        )), pos + 1))
                    }

                }
            }
        },
        _ => {

        }
    };

    anyhow::bail!("Invalid parsing");
}

fn parses_variable_assign(tokens: Vec<Tokens>, head: usize, buffer: &mut HashMap<String, String>, ) -> anyhow::Result<(Option<String>, usize)> {
    let current = &tokens[head];
    match current {
        Tokens::Var(id) => {
            let width = &tokens[head + 1];
            match width {
                Tokens::Width(size) => {
                    // An assignation
                    let assign = &tokens[head + 2];
                    
                    match assign {
                        Tokens::Assign => {
                            let (tpe, position) = parses_operation(tokens.clone(), head + 3, buffer)?;
                            
                            match tpe {
                                None => {
                                    // Variable declaration as input, add this
                                    // in the global map as a hole
                                    buffer.insert(format!("%{}", id), format!("%{}", id));
                                }
                                Some((tpe, operands) ) => {
                                    let mut subtree = String::new();
                                    subtree.push_str("(");

                                    let eterm = match (tpe.as_str(), size) {
                                        ("eq", 32) => {
                                            "i32.eq"
                                        }
                                        ("eq", 1) => {
                                            "i32.eq"
                                        }
                                        ("slt", 1) => {
                                            "i32.slt"
                                        }
                                        _ => {
                                            anyhow::bail!("Invalid conversion for {}:{}", tpe, size)
                                        }
                                    };
                                    subtree.push_str(eterm);

                                    for operand in operands {
                                        match operand {
                                            MiddleNode::LangToken(l) => {
                                                subtree.push_str(" ");
                                                subtree.push_str(&l.to_string());
                                            },
                                            MiddleNode::LookupValue(id) => {
                                                subtree.push_str(" ");
                                                subtree.push_str(&buffer[&id])
                                            },
                                        }
                                    }

                                    subtree.push_str(")");
                                    buffer.insert(format!("%{}", id), subtree);
                                }
                            }

                            return Ok((None, position));
                        },
                        _ => {
                            // Throw an error
                            anyhow::bail!("Invalid token parsing variable assignment {:?}", current)
                        }
                    }
                }
                _ => {
                    anyhow::bail!("Invalid token parsing variable assignment {:?}", current)
                }
            }
        },
        Tokens::Operator(id) => {
            match id.as_str() {
                "result" => {
                    // It is a variable declaration, add it to the global map
                    // and return if the next is EOL
                   match tokens[head + 1] {

                        Tokens::Var(idx) => {
                            let varidx = format!("%{}", idx);
                            return Ok((Some(buffer[&varidx].clone()), head + 3 /* includes the last eol token */))
                        }
                        Tokens::Integer(val) => {
                            if let Tokens::Width(w) = &tokens[head + 2] {
                                let i = match w {
                                    32 => {
                                        Lang::I32(val)
                                    }
                                    _ => {
                                        anyhow::bail!("Invalid width {}", w)
                                    }
                                };
                                println!("Returning a constant {}", i);
                                let varidx = format!("{}", i.to_string());
                                return Ok((Some(varidx), head + 4 /* includes the last eol token */))
                            }
                        }
                        _ => {
                            anyhow::bail!("Invalid root return {:?}", tokens[head + 1])
                        }
                   }
                }
                _ => anyhow::bail!("Invalid root operator {}", id)
            }
        }
        _ => {
            // Error ir other
            anyhow::bail!("Invalid token parsing the root IR {:?}", current)

        }
    }

    anyhow::bail!("Invalid parsing")
}

fn parses(tokens: Vec<Tokens>, head: usize) -> anyhow::Result<String> {
    
    let mut position = head;
    let mut expressions = HashMap::new();

    while position < tokens.len() {
        match parses_variable_assign(tokens.clone(), position, &mut expressions) {

            Ok((id, h)) => {
                
                if let Some(expr) = id {
                    return Ok(expr.clone())
                }
                println!("pos {}, len {}", h, tokens.len());
                position = h
            },
            Err(e) => {
                println!("Error {:?}", e);
                anyhow::bail!(e)
            }
        }
    }

    anyhow::bail!("Invalid souper inferring");
}


pub fn souper2Lang(rhs: &str) -> anyhow::Result<RecExpr<Lang>> {
    println!("rhs {}", rhs);


    #[derive(Debug)]
    enum LexContext {
        Start,
        Assign,
        Width,
        Var
    }


    let mut stage = LexContext::Start;
    let operators = vec![
        // Add here the operators from Souper
        "add",
        "var",
        "slt",
        "eq",
        "result"
    ];

    // TODO, move this to some lexer class

    let mut peekable = rhs.chars().peekable();
    let mut tokens = vec![];
    // Parsing here        
    while let Some(&c) = peekable.peek() {

        match c {
            ';' => {
                // Comment, discard the line :)
                break;
            }
            ' '  => {
                // Advance the head
                peekable.next();
            }
            '%' => {
                peekable.next();
                stage = LexContext::Var;
            }
            '0'..='9' => {
                peekable.next();
                // Get the number iterating the peekable, \d+
                let number = get_number(c, &mut peekable);

                match stage {
                    LexContext::Start => {
                        tokens.push(Tokens::Integer(number as i32));
                        stage = LexContext::Width;
                    }
                    LexContext::Width => {
                        //let previous_token = tokens.pop().expect("Width is not
                        //assignable");

                        tokens.push(Tokens::Width(number));
                        stage = LexContext::Start;
                     
                    },
                    LexContext::Var => {
                        // push a variable indexing token
                        // Check in the hastable if this variable was added, if
                        // so, get the subtree directly and add it as a
                        // reference
                        tokens.push(Tokens::Var(number));
                    },
                    LexContext::Assign => {
                        // It is an operator
                        // pop the last operator and assign this token :?
                        tokens.push(Tokens::Integer(number as i32));
                    },
                    _ => panic!("Invalid stage for number parsing {:?}", stage)
                }
            }
            '\n' => {
                tokens.push(Tokens::EOL);       
                peekable.next();
            }
            ',' => {
                // bypasss for now, but it can be to add an operand to the
                // current operator
                tokens.push(Tokens::Comma);       
                peekable.next();
            }
            '=' => {
                // Assign
                peekable.next();
                tokens.push(Tokens::Assign);
            }
            ':' => {
                // It is a width
                peekable.next();
                stage = LexContext::Width;
            },
            'i' => {
                // parse and integer from here if stage is Width
                peekable.next();
            }
            _ => {
                // Check if from this character some literal token can be
                // formed, if so, advance the head to it                   
                let mut mtch = false;
                for operator in operators.clone() {
                    let mut tmpeek = operator.chars().peekable();
                    if get_fixed_token(&mut peekable,&mut tmpeek) {
                        tokens.push(Tokens::Operator(format!("{}", operator)));
                        mtch = true;
                        break;
                    }
                }
                if ! mtch {                    
                    anyhow::bail!("Invalid character");
                }
            }
        }


    }

    let holes = parses(tokens, 0)?;

    // TODO, check for holes (locals, globals, mem values)
    // Read from the previous translation lang to souper to get %id -> global |
    // local | wathever_declared_var

    let r: RecExpr<Lang> = holes.parse().unwrap();

    Ok(r)
}