use std::collections::HashMap;

use parser::ast::{Node, Statements, AST, Expressions, IfExpression, BlockStatement, Identifier,
                  HashLiteral};
use evaluator::object::{Object, ObjectType, Null, Enviroment, Function, HashKey, HashType};
use buildin::{BuildIn, BuildInFunction};

const TRUE: Object = Object { object_type: ObjectType::Boolean(true) };
const FALSE: Object = Object { object_type: ObjectType::Boolean(false) };
pub const NULL: Object = Object { object_type: ObjectType::Null(Null) };

fn is_error(x: &Object) -> bool {
    x.object_type.to_type() == Object::new_error("".to_string()).object_type.to_type()
}

pub fn eval(node: AST, env: &mut Enviroment) -> Object {
    use self::AST::*;
    match node {
        Program(x) => eval_program(&x.statements, env),
        BlockStatement(x) => eval_block_statement(x, env),
        ReturnStatement(x) => {
            let val = eval(x.return_value.to_ast(), env);
            if is_error(&val) {
                return val;
            }
            Object::new_return_value(val)
        }
        LetStatement(x) => {
            let val = eval(x.value.to_ast(), env);
            if is_error(&val) {
                return val;
            }
            let result = env.set(x.name.value.clone(), val);
            result
        }
        Identifier(ref x) => eval_identifier(x, env),
        IfExpression(ref x) => eval_if_expression(x, env),
        ExpressionStatement(x) => eval(x.expression.to_ast(), env),
        IntegerLiteral(n) => Object::new_i32(n.value),
        StringLiteral(n) => Object::new_string(n.value),
        ArrayLiteral(x) => {
            let elements = eval_expression(&x.elements, env);
            match elements {
                Ok(x) => Object::new_array(x),
                Err(x) => x,
            }
        }
        HashLiteral(h) => eval_hash_literal(h, env),
        Boolean(n) => native_bool_to_boolean_obj(n.value),
        PrefixExpression(x) => {
            let operator = x.operator.clone();
            let right = eval(x.right.to_ast(), env);
            if is_error(&right) {
                return right;
            }
            eval_prefix_expression(operator, right)
        }
        InfixExpression(x) => {
            let operator = x.operator.clone();
            let left = eval(x.left.to_ast(), env);
            if is_error(&left) {
                return left;
            }
            let right = eval(x.right.to_ast(), env);
            if is_error(&right) {
                return right;
            }
            eval_infix_expression(operator, left, right)
        }
        FunctionLiteral(x) => Object::new_function(x.parameters.clone(), x.body.clone(), env),
        CallExpression(x) => {
            let func = eval(x.function.to_ast(), env);
            if is_error(&func) {
                return func;
            }
            let args = eval_expression(&x.arguments, env);
            match args {
                Ok(a) => apply_function(func, a),
                Err(x) => x,
            }
        }
        IndexExpression(z) => {
            let left = eval(z.left.to_ast(), env);
            if is_error(&left) {
                return left;
            }
            let index = eval(z.index.to_ast(), env);
            if is_error(&index) {
                return index;
            }
            eval_index_expression(left, index)
        }
    }
}

fn apply_function(func: Object, args: Vec<Object>) -> Object {
    match func.object_type {
        ObjectType::Function(f) => {
            let mut env = extend_function_env(&f, args);
            let evaluated = eval(f.body.to_enum().to_ast(), &mut env);
            unwrap_return_value(evaluated)
        }
        ObjectType::BuildIn(b) => {
            match b {
                BuildIn::Len(l) => l.call(args),
                BuildIn::PrintLn(l) => l.call(args),
            }
        }
        _ => Object::new_error(format!("not a function {:?}", func)),
    }
}

fn extend_function_env(func: &Function, args: Vec<Object>) -> Enviroment {
    let mut env = Enviroment::new_enclosed_enviroment(func.env.clone());
    for i in 0..func.parameters.len() {
        let p = &func.parameters[i];
        let a = &args[i];
        env.set(p.value.clone(), a.clone());
    }
    env
}

fn unwrap_return_value(x: Object) -> Object {
    match x.object_type {
        ObjectType::Return(x) => *x,
        _ => x,
    }
}

fn eval_expression(expressions: &Vec<Box<Expressions>>,
                   env: &mut Enviroment)
                   -> Result<Vec<Object>, Object> {
    let mut result: Vec<Object> = vec![];
    for expression in expressions.iter() {
        let evaluated = eval(expression.to_ast(), env);
        if is_error(&evaluated) {
            return Err(evaluated);
        }
        result.push(evaluated);
    }
    Ok(result)
}

fn eval_index_expression(left: Object, index: Object) -> Object {
    match left.object_type {
        ObjectType::Array(xs) => {
            if let ObjectType::Integer(i) = index.object_type {
                let max_index = xs.elements.len() - 1;
                if max_index < i as usize || i < 0 {
                    Object::new_error(format!("index out of range: max={} got={}", max_index, i))
                } else {
                    (&xs.elements)[i as usize].clone()
                }
            } else {
                Object::new_error(format!("index operator not supported {:?}", index.object_type))
            }
        }
        ObjectType::HashType(xs) => eval_hash_index_expression(xs, index),
        _ => Object::new_error(format!("index operator not supported {:?}", index.object_type)),
    }
}

fn eval_hash_index_expression(left: HashType, index: Object) -> Object {
    let maybe_key = HashKey::new(&index);
    match maybe_key {
        Some(key) => {
            match left.pairs.get(&key) {
                Some(x) => x.clone(),
                None => NULL,
            }
        }
        None => Object::new_error(format!("unusable as hash key: {:?}", index.object_type)),
    }
}

fn eval_program(statements: &Vec<Statements>, env: &mut Enviroment) -> Object {
    let mut result: Object = NULL;
    for statement in statements.iter() {
        result = eval(statement.to_ast(), env);
        if let ObjectType::Return(x) = result.object_type {
            return *x;
        }
        if let ObjectType::Error(_) = result.object_type {
            return result;
        }
    }
    result
}

fn eval_identifier(statement: &Identifier, env: &mut Enviroment) -> Object {
    match env.get(&statement.value) {
        Some(x) => x.clone(),
        None => {
            match BuildIn::set_from_string(&statement.value) {
                Some(y) => y,
                _ => Object::new_error(format!("identifier not found: {}", statement.value)),
            }
        }
    }
}

fn eval_block_statement(x: BlockStatement, env: &mut Enviroment) -> Object {
    let mut result: Object = NULL;
    for statement in x.statements.iter() {
        result = eval(statement.to_ast(), env);
        if let ObjectType::Return(_) = result.object_type {
            return result;
        }
        if let ObjectType::Error(_) = result.object_type {
            return result;
        }
    }
    result
}

fn eval_if_expression(x: &IfExpression, env: &mut Enviroment) -> Object {
    let condition = eval(x.condition.to_ast(), env);
    if is_error(&condition) {
        return condition;
    }
    match is_truthy(condition) {
        true => eval(x.consequence.to_enum().to_ast(), env),
        false => {
            if let &Some(ref y) = &x.alternative {
                return eval(y.to_enum().to_ast(), env);
            };
            NULL
        }
    }
}

fn eval_hash_literal(x: HashLiteral, env: &mut Enviroment) -> Object {
    let mut pairs: HashMap<HashKey, Object> = HashMap::new();

    for (k, v) in x.pairs.iter() {
        let key = eval(k.to_ast(), env);
        if is_error(&key) {
            return key;
        }

        let value = eval(v.to_ast(), env);
        if is_error(&value) {
            return value;
        }

        if let Some(hash_key) = HashKey::new(&key) {
            pairs.insert(hash_key, value);
        } else {
            return Object::new_error(format!("hash key not support for {:?}", key.object_type));
        }
    }

    Object { object_type: ObjectType::HashType(HashType { pairs: pairs }) }
}

fn is_truthy(x: Object) -> bool {
    match x {
        NULL => false,
        TRUE => true,
        FALSE => false,
        _ => true,
    }
}

fn native_bool_to_boolean_obj(x: bool) -> Object {
    match x {
        true => TRUE,
        false => FALSE,
    }
}

fn eval_infix_expression(operator: String, left: Object, right: Object) -> Object {
    if let ObjectType::Integer(l) = left.object_type {
        if let ObjectType::Integer(r) = right.object_type {
            return eval_integer_infix_expression(operator, l, r);
        }
    }

    if let ObjectType::StringType(l) = left.object_type.clone() {
        if let ObjectType::StringType(r) = right.object_type.clone() {
            return eval_string_infix_expression(operator, l, r);
        }
    }

    if left.object_type.to_type() != right.object_type.to_type() {
        return Object::new_error(format!("type mismatch: {:?} {} {:?}",
                                         left.object_type,
                                         operator,
                                         right.object_type));
    }

    match operator.as_str() {
        "==" => native_bool_to_boolean_obj(left == right),
        "!=" => native_bool_to_boolean_obj(left != right),
        _ => {
            Object::new_error(format!("unknown operator: {:?} {} {:?}",
                                      left.object_type,
                                      operator,
                                      right.object_type))
        }
    }
}

fn eval_integer_infix_expression(operator: String, left: i32, right: i32) -> Object {
    match operator.as_str() {
        "+" => Object::new_i32(left + right),
        "-" => Object::new_i32(left - right),
        "*" => Object::new_i32(left * right),
        "/" => Object::new_i32(left / right),
        "<" => native_bool_to_boolean_obj(left < right),
        ">" => native_bool_to_boolean_obj(left > right),
        "==" => native_bool_to_boolean_obj(left == right),
        "!=" => native_bool_to_boolean_obj(left != right),
        _ => Object::new_error(format!("unknown operator: Integer {} Integer", operator)),
    }
}

fn eval_string_infix_expression(operator: String, left: String, right: String) -> Object {
    match operator.as_str() {
        "+" => Object::new_string(format!("{}{}", left, right)),
        _ => Object::new_error(format!("unknown operator: String {} String", operator)),
    }
}

fn eval_prefix_expression(operator: String, right: Object) -> Object {
    match operator.as_str() {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_operator_expression(right),
        _ => Object::new_error(format!("unknown operator: {}{:?}", operator, right.object_type)),
    }
}

fn eval_minus_operator_expression(right: Object) -> Object {
    match right.object_type {
        ObjectType::Integer(x) => Object::new_i32(-x),
        _ => Object::new_error(format!("unknown operator: -{:?}", right.object_type)),
    }
}

fn eval_bang_operator_expression(right: Object) -> Object {
    match right {
        TRUE => FALSE,
        FALSE => TRUE,
        NULL => TRUE,
        _ => FALSE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::lexer;
    use parser::parser;
    use parser::ast::Node;

    fn test_eval(input: String) -> Object {
        let l = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(l);
        let program = parser.parse_program();
        let mut env = Enviroment::new();
        eval(program.to_enum().to_ast(), &mut env)
    }

    #[test]
    fn it_should_evaluate_integer_expression() {
        let expects = [("5", 5),
                       ("10", 10),
                       ("-5", -5),
                       ("-10", -10),
                       ("5 + 5 + 5 + 5 - 10", 10),
                       ("2 * 2 * 2 * 2 * 2", 32),
                       ("-50 + 100 + -50", 0),
                       ("5 * 2 + 10", 20),
                       ("5 + 2 * 10", 25),
                       ("20 + 2 * -10", 0),
                       ("50 / 2 * 2 + 10", 60),
                       ("2 * (5 + 10)", 30),
                       ("3 * 3 * 3 + 10", 37),
                       ("3 * (3 * 3) + 10", 37),
                       ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_string_expression() {
        let expects = [("\"hello world\"", "hello world")];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_string().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_concatenation_string() {
        let expects = [("\"hello\" + \" \" + \"world\"", "hello world")];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_string().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_boolean_expression() {
        let expects = [("true", true),
                       ("false", false),
                       ("1 < 2", true),
                       ("1 > 2", false),
                       ("1 < 1", false),
                       ("1 > 1", false),
                       ("1 == 1", true),
                       ("1 != 1", false),
                       ("1 == 2", false),
                       ("1 != 2", true),
                       ("true == true", true),
                       ("false == false", true),
                       ("true == false", false),
                       ("true != false", true),
                       ("false != true", true),
                       ("(1 < 2) == true", true),
                       ("(1 < 2) == false", false),
                       ("(1 > 2) == true", false),
                       ("(1 > 2) == false", true)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_bool().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_bang_operator() {
        let expects = [("!true", false), ("!false", true)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_bool().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_if_else_expression() {
        let expects = [("if (true) { 10 }", Some(10)),
                       ("if (false) { 10 }", None),
                       ("if (1) { 11 }", Some(11)),
                       ("if (1 < 2) { 12 }", Some(12)),
                       ("if (1 > 2) { 10 }", None),
                       ("if (1 > 2) { 10 } else { 20 }", Some(20)),
                       ("if (1 < 2) { 13 } else { 20 }", Some(13))];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_return_expression() {
        let expects = [("return 10;", 10),
                       ("return 10; 9;", 10),
                       ("return 2 * 5; 9;", 10),
                       ("9; return 2 * 5; 9;", 10),
                       ("
                        if (10 > 1) {
                           if (10 > 1) {
                               return 10;
                           }
                        }
                        return 1;
                       ",
                        10)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_error_handling() {
        let expects = [("5 + true;", "type mismatch: Integer(5) + Boolean(true)"),
                       ("5 + true; 5;", "type mismatch: Integer(5) + Boolean(true)"),
                       ("-true;", "unknown operator: -Boolean(true)"),
                       ("true + false;", "unknown operator: Boolean(true) + Boolean(false)"),
                       ("5; true + false; 5;", "unknown operator: Boolean(true) + Boolean(false)"),
                       ("if (10 > 1) { true + false; };",
                        "unknown operator: Boolean(true) + Boolean(false)"),
                       ("
                       if (10 > 1) {
                            if (10 > 1) {
                                return true + false;
                            };
                        };
                       ",
                        "unknown operator: Boolean(true) + Boolean(false)"),
                       ("foobar", "identifier not found: foobar"),
                       ("\"hello world\" - \"world\"", "unknown operator: String - String")];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_error_message().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_let_statements() {
        let expects = [("let a = 5; a;", 5),
                       ("let a = 5 * 5; a;", 25),
                       ("let a = 5; let b = a; b;", 5),
                       ("let a = 5; let b = a; let c = a + b + 5; c;", 15)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_function_literal() {
        let expects = [("let identity = fn(x) {x;}; identity(5);", 5),
                       ("let identity = fn(x) {return x;}; identity(5);", 5),
                       ("let double = fn(x) {x * 2;}; double(5);", 10),
                       ("let add = fn(x, y) {x + y;}; add(5, 5);", 10),
                       ("let add = fn(x, y) {x + y;}; add(5 + 5, add(5, 5));", 20),
                       ("fn(x) {x;}(5)", 5)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_array_literal() {
        let expects = [("[1, 2, 3][0]", 1),
                       ("[1, 2, 3][1]", 2),
                       ("[1, 2, 3][2]", 3),
                       ("let i = 0; [1][i]", 1),
                       ("[1, 2, 3][1 + 1]", 3),
                       ("let myArray = [1, 2, 3]; myArray[2]", 3),
                       ("let a = 0;let b = 0; a + -b", 0),
                       ("let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];", 6),
                       ("let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]", 2)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_array_error() {
        let expects = [("[1, 2, 3][3]", "index out of range: max=2 got=3"),
                       ("[1, 2, 3][-1]", "index out of range: max=2 got=-1")];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_error_message().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_array_index_expression() {
        let result = test_eval("[1, 2 * 2, 3 + 3]".to_string());
        if let ObjectType::Array(x) = result.object_type {
            assert_eq!(x.elements.len(), 3);
            assert_eq!(x.elements[0].to_i32().unwrap(), 1);
            assert_eq!(x.elements[1].to_i32().unwrap(), 4);
            assert_eq!(x.elements[2].to_i32().unwrap(), 6);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_should_call_build_in_functions() {
        let expects = [("len(\"\");", 0), ("len(\"four\");", 4), ("len(\"hello world\");", 11)];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32().unwrap(), expect.1);
        }

        let error_expects = [("len(1);", "argument to \"len\" not supported. got Integer(1)"),
                             ("len(\"one\", \"two\");", "wrong number of arguments. got 2 want=1")];
        for expect in error_expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_error_message().unwrap(), expect.1);
        }
    }

    #[test]
    fn it_should_evaluate_hash_literal() {
        let result = test_eval("
        let x = {
            \"one\": 10 - 9,
            \"two\": 1 + 1,
            \"thr\" + \"ee\": 6 / 2,
            4: 4,
            true: 5,
            false: 6
        };
        "
                                       .to_string());
        if let ObjectType::HashType(x) = result.object_type {
            let key_1 = HashKey::StringType("one".to_string());
            let key_2 = HashKey::StringType("two".to_string());
            let key_3 = HashKey::StringType("three".to_string());
            let key_4 = HashKey::Integer(4 as i32);
            let key_5 = HashKey::Boolean(true);
            let key_6 = HashKey::Boolean(false);

            let value_1 = Object::new_i32(1);
            let value_2 = Object::new_i32(2);
            let value_3 = Object::new_i32(3);
            let value_4 = Object::new_i32(4);
            let value_5 = Object::new_i32(5);
            let value_6 = Object::new_i32(6);

            assert_eq!(*x.pairs.get(&key_1).unwrap(), value_1);
            assert_eq!(*x.pairs.get(&key_2).unwrap(), value_2);
            assert_eq!(*x.pairs.get(&key_3).unwrap(), value_3);
            assert_eq!(*x.pairs.get(&key_4).unwrap(), value_4);
            assert_eq!(*x.pairs.get(&key_5).unwrap(), value_5);
            assert_eq!(*x.pairs.get(&key_6).unwrap(), value_6);

        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_should_evaluate_hash_index_expression() {
        let expects = [("{\"foo\": 5}[\"foo\"]", Some(5)),
                       ("{\"foo\": 5}[\"bar\"]", None),
                       ("let key = \"foo\"; {\"foo\": 5}[key]", Some(5)),
                       ("{}[\"bar\"]", None),
                       ("{5: 5}[5]", Some(5)),
                       ("{true: 5}[true]", Some(5)),
                       ("{false: 5}[false]", Some(5))];
        for expect in expects.iter() {
            let result = test_eval(expect.0.to_string());
            assert_eq!(result.to_i32(), expect.1);
        }
    }
}

