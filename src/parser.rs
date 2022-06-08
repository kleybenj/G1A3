use crate::{C1Lexer, C1Token, ParseResult};
use crate::C1Token::{And, Assign, Asterisk, ConstBoolean, ConstFloat, ConstInt, Equal, Greater, GreaterEqual, Identifier, KwBoolean, KwFloat, KwIf, KwInt, KwPrintf, KwReturn, KwVoid, LeftBrace, LeftParenthesis, Less, LessEqual, Minus, NotEqual, Or, Plus, RightBrace, RightParenthesis, Semicolon, Slash};

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
        Some(KwIf) => parse_if(lexer), ///ifstatement
        Some(KwReturn) => {parse_return(lexer)?; ///returnstatement
            check_token(lexer, Semicolon)}, /// ";"
        Some(KwPrintf) => {parse_printf(lexer)?; ///printf
            check_token(lexer, Semicolon)}, /// ";"
        Some(Identifier) =>
            if lookahead == Some(Assign) { ///statassignment ";"
                parse_statassignment(lexer)?;
                check_token(lexer, Semicolon)}
            else if lookahead == Some(LeftParenthesis) {parse_functioncall(lexer)?; check_token(lexer, Semicolon)} ///funccall ";"
            else { Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) },
        _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) /// none of the above
    }
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
    Ok(())
}

/// printf              ::= <KW_PRINTF> "(" assignment ")"
fn parse_printf(mut lexer: &C1Lexer) -> ParseResult{
    check_token(lexer, KwPrintf)?; /// <KW_PRINTF>
    check_token(lexer, LeftParenthesis)?; /// "("
    parse_assignment(lexer)?; /// assignment
    check_token(lexer, RightParenthesis) /// "("
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
    if lexer.current_token() == Some(Identifier) { ///( <ID> "=" assignment )
        check_token(lexer,Identifier)?; /// <ID>
        check_token(lexer, Assign)?; /// "="
        parse_assignment(lexer) /// assignment
    }
    else { /// | expr
        parse_expression(lexer)
    }
}

/// expr                ::= simpexpr ( ( "==" | "!=" | "<=" | ">=" | "<" | ">" ) simpexpr )?
fn parse_expression(mut lexer: &C1Lexer) -> ParseResult{
    parse_simpexpr(lexer)?; ///simpexpr
    /// If the next part exists, it defenitely starts with one of the following: ( "==" | "!=" | "<=" | ">=" | "<" | ">" )
    match lexer.current_token() {
        Some(Equal) => {check_token(lexer, Equal)?;
            parse_simpexpr(lexer)}, /// "==" simpexpr
        Some(NotEqual) => {check_token(lexer, NotEqual)?;
            parse_simpexpr(lexer)}, /// "!=" simpexpr
        Some(LessEqual) => {check_token(lexer, LessEqual)?;
            parse_simpexpr(lexer)}, /// "<=" simpexpr
        Some(GreaterEqual) => {check_token(lexer, GreaterEqual)?;
            parse_simpexpr(lexer)}, /// ">=" simpexpr
        Some(Less) => {check_token(lexer, Less)?;
            parse_simpexpr(lexer)}, /// "<" simpexpr
        Some(Greater) => {check_token(lexer, Greater)?;
            parse_simpexpr(lexer)}, /// ">" simpexpr
        _ => Ok(()) /// there was no second part
    }
}

/// simpexpr            ::= ( "-" )? term ( ( "+" | "-" | "||" ) term )*
fn parse_simpexpr(mut lexer: &C1Lexer) -> ParseResult{
    if lexer.current_token() == Some(Minus) { /// ( "-" )?
        check_token(lexer, Minus)?;
    }
    parse_term(lexer); /// term
    while lexer.current_token() == Some(Plus) | Some(Minus) | Some(Or) { /// ( ( "+" | "-" | "||" ) term )*
        match lexer.current_token() {
            Some(Plus) => {check_token(lexer, Plus)?; /// "+" term
                parse_term(lexer)?;},
            Some(Minus) => {check_token(lexer, Minus)?; /// "-" term
                parse_term(lexer)?;},
            Some(Or) => {check_token(lexer, Or)?; /// "||" term
                parse_term(lexer)?;},
            _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) ///this should never occur since we explicitely checked for it before entering the match
        }
    }
    Ok(()) ///no last part
}

/// term                ::= factor ( ( "*" | "/" | "&&" ) factor )*
fn parse_term(mut lexer: &C1Lexer) -> ParseResult{
    parse_factor(lexer)?; /// factor
    while lexer.current_token() == Some(Asterisk) | Some(Slash) | Some(And) { /// ( ( "*" | "/" | "&&" ) factor )*
        match lexer.current_token() {
            Some(Asterisk) => {check_token(lexer, Asterisk)?; /// "*" factor
                parse_factor(lexer)?;}
            Some(Slash) => {check_token(lexer, Slash)?; /// "/" factor
                parse_factor(lexer)?;}
            Some(And) => {check_token(lexer, And)?; /// "&&" factor
                parse_factor(lexer)?;}
            _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) ///this should never occur since we explicitely checked for it before entering the match
        }
    }
    Ok(()) /// no last part
}

///factor              ::= <CONST_INT>
//                       | <CONST_FLOAT>
//                       | <CONST_BOOLEAN>
//                       | functioncall
//                       | <ID>
//                       | "(" assignment ")"

fn parse_factor(mut lexer: &C1Lexer) -> ParseResult{
    match lexer.current_token() {
        Some(ConstInt) => check_token(lexer, ConstInt), /// <CONST_INT>
        Some(ConstFloat) => check_token(lexer, ConstFloat), /// <CONST_FLOAT>
        Some(ConstBoolean) => check_token(lexer, ConstBoolean), /// <CONST_BOOLEAN>
        Some(Identifier) => if lexer.peek_token() == Some(LeftParenthesis) { ///this might be ambiguous? there should never be a "(" if it's just the <ID>, but I'm not sure. ||| functioncall | <ID>
            parse_functioncall(lexer) /// functioncall
        }
        else {
            check_token(lexer, Identifier) /// <ID>
        },
        Some(LeftParenthesis) => {check_token(lexer, LeftParenthesis)?; parse_assignment(lexer)?; check_token(lexer, RightParenthesis)} /// "(" assignment ")"
        _ => Err(format!("Error found: {:?} at {:?}", lexer.current_text(), lexer.current_line_number())) ///not a factor
    }
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