use crate::{C1Lexer, C1Token, ParseResult};
use crate::C1Token::{Assign, Identifier, KwBoolean, KwFloat, KwIf, KwInt, KwPrintf, KwReturn, KwVoid, LeftBrace, LeftParenthesis, RightBrace, RightParenthesis, Semicolon};

/// program ::= ( functiondefinition )* <EOF>
pub fn parse(input: &str) -> ParseResult {
    let mut lexer = C1Lexer::new(input)?;
    if lexer.peek_token() != EOF {
        parse_funcdef(lexer)?;
    }
    else {
        Err(format!("File is empty!"))
    }
    while lexer.peek_token() != EOF { ///
        parse_funcdef(lexer)?;
    }
    Ok(())
}
/// functiondefinition  ::= type <ID> "(" ")" "{" statementlist "}"
fn parse_funcdef(mut lexer: &C1Lexer) -> ParseResult {
    parse_type(lexer)?; ///type
    check_token(lexer, Identifier)?; /// <ID>
    check_token(lexer, LeftParenthesis)?; /// "("
    check_token(lexer, RightParenthesis)?; /// "("
    parse_statementlist(&lexer)?; /// "{" statementlist "}"
    Ok(())
}
/// functioncall ::= <ID> "(" ")"
fn parse_functioncall(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, Identifier)?; /// <ID>
    check_token(lexer, LeftParenthesis)?; /// "("
    check_token(lexer, RightParenthesis)?; /// ")"
    Ok(())
}

/// statementlist       ::= ( block )*
/// statementlists are always surrounded by "{" and "}", we will use this to find out how often a block has to be parsed
fn parse_statementlist(mut lexer: &C1Lexer) -> ParseResult{
    if lexer.current_token() == Some(LeftBrace){ /// check whether there is actually an opening brace, otherwise we won't go into the while loop
        while lexer.current_token() == Some(LeftBrace){
            check_token(lexer, LeftBrace)?; /// "{"
            /// there doesn't have to be a block (see the '*'). In this case there are just two empty braces like so: '{}'
            if lexer.current_token() != Some(RightBrace) {
                parse_block(lexer)?;
            }
            check_token(lexer,RightBrace)?; /// "}"
        }
    }
    else { /// if there wasn't an opening brace there is an error
        Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number()))
    }
    Ok(())
}

/// block               ::= "{" statementlist "}" | statement
fn parse_block(mut lexer: &C1Lexer) -> ParseResult{
    if lexer.current_token() == LeftBrace { /// "{" statementlist "}"
        parse_statementlist(lexer)?;
    }
    else {
        parse_statement(lexer)?; ///statement
    }
    Ok(())
}

///statement           ::= ifstatement
///                       | returnstatement ";"
///                       | printf ";"
///                       | statassignment ";"
///                       | functioncall ";"
fn parse_statement(mut lexer: &C1Lexer) -> ParseResult{
    let lookahead = lexer.peek_token();
    let current = lexer.current_token();
    match current {
        Some(KwIf) => parse_if(lexer)?, ///ifstatement
        Some(KwReturn) => parse_return(lexer)?, ///returnstatement ";"
        Some(KwPrintf) => parse_printf(lexer)?, ///printf ";"
        Some(Identifier) =>
            if lookahead == Some(Assign) { ///statassignment ";"
                parse_statassignment(lexer)?;
                check_token(lexer, Semicolon)?;}
            else if lookahead == Some(LeftParenthesis) {parse_functioncall(lexer)?; check_token(lexer, Semicolon)?;} ///funccall ";"
            else { Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) },
        _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) /// none of the above
    }
    Ok(())
}
/// ifstatement         ::= <KW_IF> "(" assignment ")" block
fn parse_if(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, KwIf)?; /// <KW_IF>>
    check_token(lexer, LeftParenthesis)?; /// "("
    parse_assignment(lexer)?; /// assignment
    check_token(lexer, RightParenthesis)?; /// ")"
    parse_block(lexer)?; /// block
    Ok(())
}
/// returnstatement     ::= <KW_RETURN> ( assignment )?
/// returnstatements are always followed by a ';', so this way we can check for the '?'
fn parse_return(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, KwReturn)?; /// <KW_RETURN>
    if lexer.current_token() != Some(Semicolon){ /// ( assignment )?
        parse_assignment(lexer)?;
    }
    check_token(lexer, Semicolon) /// ";"
}

/// printf              ::= <KW_PRINTF> "(" assignment ")"
fn parse_printf(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, KwPrintf)?; /// <KW_PRINTF>
    check_token(lexer, LeftParenthesis)?; /// "("
    parse_assignment(lexer)?; /// assignment
    check_token(lexer, RightParenthesis)?; /// "("
    check_token(lexer, Semicolon) /// ";"
}
///type                ::= <KW_BOOLEAN>
//                       | <KW_FLOAT>
//                       | <KW_INT>
//                       | <KW_VOID>
fn parse_type(mut lexer: &C1Lexer) -> ParseResult{
    let current = lexer.current_token();
    match current {
        Some(KwBoolean) => check_token(lexer, KwBoolean)?, ///<KW_BOOLEAN>
        Some(KwFloat) => check_token(lexer, KwFloat)?, ///<KW_FLOAT>
        Some(KwInt) => check_token(lexer, KwInt)?, ///<KW_INT>
        Some(KwVoid) => check_token(lexer, KwVoid)?, ///<KW_VOID>
        _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) ///None of the above
    }
    Ok(())
}
/// statassignment      ::= <ID> "=" assignment
fn parse_statassignment(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, Identifier)?; /// <ID>
    check_token(lexer, Assign)?; /// "="
    parse_assignment(lexer) /// assignment
}

/// assignment          ::= ( ( <ID> "=" assignment ) | expr )
fn parse_assignment(mut lexer: &C1Lexer) -> ParseResult{

}

/// expr                ::= simpexpr ( ( "==" | "!=" | "<=" | ">=" | "<" | ">" ) simpexpr )?
fn parse_expression(mut lexer: &C1Lexer) -> ParseResult{

}

/// simpexpr            ::= ( "-" )? term ( ( "+" | "-" | "||" ) term )*
fn parse_simpexpr(mut lexer: &C1Lexer) -> ParseResult{

}

/// term                ::= factor ( ( "*" | "/" | "&&" ) factor )*
fn parse_term(mut lexer: &C1Lexer) -> ParseResult{

}

///factor              ::= <CONST_INT>
//                       | <CONST_FLOAT>
//                       | <CONST_BOOLEAN>
//                       | functioncall
//                       | <ID>
//                       | "(" assignment ")"

fn parse_factor(mut lexer: &C1Lexer) -> ParseResult{

}

fn check_token(mut lexer: &C1Lexer, token: C1Token) -> ParseResult{ ///check_and_eat
    if lexer.current_token() == Some(token) { /// "}"
        lexer.eat();
    }
    else {
        Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number()))
    }
    Ok(())
}