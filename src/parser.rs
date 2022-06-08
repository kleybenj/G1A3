use crate::{C1Lexer, C1Token, ParseResult};
use crate::C1Token::{And, Assign, Asterisk, ConstBoolean, ConstFloat, ConstInt, Equal, Error, Greater, GreaterEqual, Identifier, KwBoolean, KwFloat, KwIf, KwInt, KwPrintf, KwReturn, KwVoid, LeftBrace, LeftParenthesis, Less, LessEqual, Minus, NotEqual, Or, Plus, RightBrace, RightParenthesis, Semicolon, Slash};

pub struct C1Parser<'a> {
    lexer: C1Lexer<'a>,
}

impl<'a> C1Parser<'a> {
    pub fn parse(input: &str) -> ParseResult {
        let mut lexer = C1Lexer::new(input);
        let mut parser = C1Parser::new(lexer);
        parser.parse_program()
    }

    pub fn new(lexer: C1Lexer<'a>) -> Self {
        Self {
            lexer
        }
    }
    /// program ::= ( functiondefinition )* <EOF>
    pub fn parse_program(mut self) -> ParseResult {
        if self.peek_token() != Some(Error) {
            self.parse_funcdef()?;
        } else {
            return Err(format!("File is empty!"));
        }
        while self.peek_token() != None {
            self.parse_funcdef()?;
        }
        Ok(())
    }

    /// functiondefinition  ::= type <ID> "(" ")" "{" statementlist "}"
    fn parse_funcdef(&mut self) -> ParseResult {
        self.parse_type()?; //type
        self.check_token(Identifier)?; // <ID>
        self.check_token(LeftParenthesis)?; // "("
        self.check_token(RightParenthesis)?; // "("
        self.check_token(LeftBrace)?; //"{"
        self.parse_statementlist()?; //  statementlist
        self.check_token(RightBrace) //"}"
    }
    /// functioncall ::= <ID> "(" ")"
    fn parse_functioncall(&mut self) -> ParseResult {
        self.check_token(Identifier)?; // <ID>
        self.check_token(LeftParenthesis)?; // "("
        self.check_token(RightParenthesis) // ")"
    }

    /// statementlist       ::= ( block )*
    /// statementlists are always surrounded by "{" and "}", we will use this to find out how often a block has to be parsed
    /// there doesn't have to be a block (see the '*'). In this case there are just two empty braces like so: '{}'
    fn parse_statementlist(&mut self) -> ParseResult {
        while self.lexer.current_token() != Some(RightBrace) {
            self.parse_block()?;
        }
        Ok(())
    }

    /// block               ::= "{" statementlist "}" | statement
    fn parse_block(&mut self) -> ParseResult {
        if self.lexer.current_token() == Some(LeftBrace) { // "{" statementlist "}"
            self.check_token(LeftBrace)?;
            self.parse_statementlist()?;
            self.check_token(RightBrace)
        } else {
            self.parse_statement() //statement
        }
    }

    ///statement           ::= ifstatement
    ///                       | returnstatement ";"
    ///                       | printf ";"
    ///                       | statassignment ";"
    ///                       | functioncall ";"
    fn parse_statement(&mut self) -> ParseResult {
        let lookahead = self.peek_token();
        let current = self.lexer.current_token();
        match current {
            Some(KwIf) => self.parse_if(), //ifstatement
            Some(KwReturn) => {
                self.parse_return()?; //returnstatement
                self.check_token(Semicolon)
            } // ";"
            Some(KwPrintf) => {
                self.parse_printf()?; //printf
                self.check_token(Semicolon)
            } // ";"
            Some(Identifier) =>
                if lookahead == Some(Assign) { //statassignment ";"
                    self.parse_statassignment()?;
                    self.check_token(Semicolon)
                } else if lookahead == Some(LeftParenthesis) {  //funccall ";"
                    self.parse_functioncall()?;
                    self.check_token(Semicolon)
                } else { return Err(format!("Error found while trying to parse statement: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())); },
            _ => return Err(format!("Error found statement return error: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())) // none of the above
        }
    }

    /// ifstatement         ::= <KW_IF> "(" assignment ")" block
    fn parse_if(&mut self) -> ParseResult {
        self.check_token(KwIf)?; // <KW_IF>>
        self.check_token(LeftParenthesis)?; // "("
        self.parse_assignment()?; // assignment
        self.check_token(RightParenthesis)?; // ")"
        self.parse_block()?; // block
        Ok(())
    }
    /// returnstatement     ::= <KW_RETURN> ( assignment )?
    /// returnstatements are always followed by a ';', so this way we can check for the '?'
    fn parse_return(&mut self) -> ParseResult {
        self.check_token(KwReturn)?; // <KW_RETURN>
        if self.lexer.current_token() != Some(Semicolon) { // ( assignment )?
            self.parse_assignment()?;
        }
        Ok(())
    }

    /// printf              ::= <KW_PRINTF> "(" assignment ")"
    fn parse_printf(&mut self) -> ParseResult {
        self.check_token(KwPrintf)?; // <KW_PRINTF>
        self.check_token(LeftParenthesis)?; // "("
        self.parse_assignment()?; // assignment
        self.check_token(RightParenthesis) // "("
    }

    ///type                ::= <KW_BOOLEAN>
//                       | <KW_FLOAT>
//                       | <KW_INT>
//                       | <KW_VOID>
    fn parse_type(&mut self) -> ParseResult {
        let current = self.lexer.current_token();
        match current {
            Some(KwBoolean) => self.check_token(KwBoolean)?, //<KW_BOOLEAN>
            Some(KwFloat) => self.check_token(KwFloat)?, //<KW_FLOAT>
            Some(KwInt) => self.check_token(KwInt)?, //<KW_INT>
            Some(KwVoid) => self.check_token(KwVoid)?, //<KW_VOID>
            _ => return Err(format!("Error found: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())) //None of the above
        }
        Ok(())
    }
    /// statassignment      ::= <ID> "=" assignment
    fn parse_statassignment(&mut self) -> ParseResult {
        self.check_token(Identifier)?; // <ID>
        self.check_token(Assign)?; // "="
        self.parse_assignment() // assignment
    }

    /// assignment          ::= ( ( <ID> "=" assignment ) | expr )
    /// This sucks because there is also a case where an expr starts with a simpexpr which might start with a term which might begin with an ID
    fn parse_assignment(&mut self) -> ParseResult {
        if self.lexer.current_token() == Some(Identifier) && self.peek_token() == Some(Assign) { //( <ID> "=" assignment )
            self.check_token(Identifier)?; // <ID>
            self.check_token(Assign)?; // "="
            self.parse_assignment() // assignment
        } else { // | expr
            self.parse_expression()
        }
    }

    /// expr                ::= simpexpr ( ( "==" | "!=" | "<=" | ">=" | "<" | ">" ) simpexpr )?
    fn parse_expression(&mut self) -> ParseResult {
        self.parse_simpexpr()?; //simpexpr
        // If the next part exists, it defenitely starts with one of the following: ( "==" | "!=" | "<=" | ">=" | "<" | ">" )
        match self.lexer.current_token() {
            Some(Equal) => {
                self.check_token(Equal)?;
                self.parse_simpexpr()
            } // "==" simpexpr
            Some(NotEqual) => {
                self.check_token(NotEqual)?;
                self.parse_simpexpr()
            } // "!=" simpexpr
            Some(LessEqual) => {
                self.check_token(LessEqual)?;
                self.parse_simpexpr()
            } // "<=" simpexpr
            Some(GreaterEqual) => {
                self.check_token(GreaterEqual)?;
                self.parse_simpexpr()
            } // ">=" simpexpr
            Some(Less) => {
                self.check_token(Less)?;
                self.parse_simpexpr()
            } // "<" simpexpr
            Some(Greater) => {
                self.check_token(Greater)?;
                self.parse_simpexpr()
            } // ">" simpexpr
            _ => Ok(()) // there was no second part
        }
    }

    /// simpexpr            ::= ( "-" )? term ( ( "+" | "-" | "||" ) term )*
    fn parse_simpexpr(&mut self) -> ParseResult {
        if self.lexer.current_token() == Some(Minus) { // ( "-" )?
            self.check_token(Minus)?;
        }
        self.parse_term()?; // term
        let mut whilecondition = self.whilechecker(C1Token::Plus, C1Token::Minus, C1Token::Or);
        while whilecondition == true { // ( ( "+" | "-" | "||" ) term )*
            match self.lexer.current_token() {
                Some(Plus) => {
                    self.check_token(Plus)?; // "+" term
                    self.parse_term()?;
                }
                Some(Minus) => {
                    self.check_token(Minus)?; // "-" term
                    self.parse_term()?;
                }
                Some(Or) => {
                    self.check_token(Or)?; // "||" term
                    self.parse_term()?;
                }
                _ => return Err(format!("Error found: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())) //this should never occur since we explicitely checked for it before entering the match
            }
            whilecondition = self.whilechecker(C1Token::Plus, C1Token::Minus, C1Token::Or);
        }
        Ok(()) //no last part
    }

    /// term                ::= factor ( ( "*" | "/" | "&&" ) factor )*
    fn parse_term(&mut self) -> ParseResult {
        self.parse_factor()?; // factor
        let mut whilecondition = self.whilechecker(C1Token::Asterisk, C1Token::Slash, C1Token::And);
        while whilecondition == true { // ( ( "*" | "/" | "&&" ) factor )*
            match self.lexer.current_token() {
                Some(Asterisk) => {
                    self.check_token(Asterisk)?; // "*" factor
                    self.parse_factor()?;
                }
                Some(Slash) => {
                    self.check_token(Slash)?; // "/" factor
                    self.parse_factor()?;
                }
                Some(And) => {
                    self.check_token(And)?; // "&&" factor
                    self.parse_factor()?;
                }
                _ => return Err(format!("Error found while parsing term: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())) //this should never occur since we explicitely checked for it before entering the match
            }
            whilecondition = self.whilechecker(C1Token::Asterisk, C1Token::Slash, C1Token::And);
        }
        Ok(()) // no last part
    }

    ///factor              ::= <CONST_INT>
//                       | <CONST_FLOAT>
//                       | <CONST_BOOLEAN>
//                       | functioncall
//                       | <ID>
//                       | "(" assignment ")"

    fn parse_factor(&mut self) -> ParseResult {
        match self.lexer.current_token() {
            Some(ConstInt) => self.check_token(ConstInt), // <CONST_INT>
            Some(ConstFloat) => self.check_token(ConstFloat), // <CONST_FLOAT>
            Some(ConstBoolean) => self.check_token(ConstBoolean), // <CONST_BOOLEAN>
            Some(Identifier) => if self.peek_token() == Some(LeftParenthesis) { //this might be ambiguous? there should never be a "(" if it's just the <ID>, but I'm not sure. ||| functioncall | <ID>
                self.parse_functioncall() // functioncall
            } else {
                self.check_token(Identifier) // <ID>
            },
            Some(LeftParenthesis) => {
                self.check_token(LeftParenthesis)?;
                self.parse_assignment()?;
                self.check_token(RightParenthesis)
            } // "(" assignment ")"
            _ => return Err(format!("Error found while parsing factor: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number())) //not a factor
        }
    }
    ///check_and_eat
    fn check_token(&mut self, token: C1Token) -> ParseResult {
        if self.lexer.current_token() == Some(token) { // "}"
            self.lexer.eat();
        } else {
            return Err(format!("Error found in eat function: {:?} at {:?}", self.lexer.current_text(), self.lexer.current_line_number()));
        }
        Ok(())
    }

    fn peek_token(&self) -> Option<C1Token> {
        self.lexer.peek_token()
    }
    fn whilechecker(&self, token: C1Token, token2: C1Token, token3: C1Token) -> bool {
        return if self.lexer.current_token() == Some(token) {
            true
        } else if self.lexer.current_token() == Some(token2) {
            true
        } else if self.lexer.current_token() == Some(token3) {
            true
        } else { false }
    }
}