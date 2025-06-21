use clap::Parser as ClapParser;
use std::{
    fs::{self},
    io,
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,  // {
    RightBrace, // }
    String(String),
    Number(f64),
    True,
    False,
    Null,
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Comma,        // ,
    Whitespace,
    Eof,
}

#[derive(Debug)]
struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// 当前位置是空白字符则跳过
    fn skio_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1
        }
    }

    fn next_token(&mut self) -> Token {
        self.skio_whitespace();

        if self.pos >= self.input.len() {
            return Token::Eof;
        }

        let ch = self.input[self.pos];
        self.pos += 1;

        match ch {
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '"' => self.read_string(),
            't' => self.read_true(),
            'f' => self.read_false(),
            'n' => self.read_null(),
            '0'..='9' | '-' => self.read_number(ch),
            _ => Token::Whitespace,
        }
    }

    fn read_string(&mut self) -> Token {
        let mut result = String::new();
        while self.pos < self.input.len() && self.input[self.pos] != '"' {
            result.push(self.input[self.pos]);
            self.pos += 1;
        }
        if self.pos < self.input.len() && self.input[self.pos] == '"' {
            self.pos += 1;
        }

        Token::String(result)
    }

    fn read_true(&mut self) -> Token {
        if self.pos + 3 <= self.input.len()
            && self.input[self.pos - 1..self.pos + 3] == ['t', 'r', 'u', 'e']
        {
            self.pos += 3;
            Token::True
        } else {
            Token::Whitespace
        }
    }

    fn read_false(&mut self) -> Token {
        if self.pos + 4 <= self.input.len()
            && self.input[self.pos - 1..self.pos + 4] == ['f', 'a', 'l', 's', 'e']
        {
            self.pos += 4;
            Token::False
        } else {
            Token::Whitespace
        }
    }

    fn read_null(&mut self) -> Token {
        if self.pos + 3 <= self.input.len()
            && self.input[self.pos - 1..self.pos + 3] == ['n', 'u', 'l', 'l']
        {
            self.pos += 3;
            Token::Null
        } else {
            Token::Whitespace
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut number = first.to_string();
        while self.pos < self.input.len()
            && (self.input[self.pos].is_digit(10) || self.input[self.pos] == '.')
        {
            number.push(self.input[self.pos]);
            self.pos += 1;
        }
        Token::Number(number.parse().unwrap_or(0.0))
    }
}

#[derive(Debug)]
enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input".into());
        }
        match &self.tokens[self.pos] {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(s) => {
                self.pos += 1;
                Ok(JsonValue::String(s.clone()))
            }
            Token::Number(n) => {
                self.pos += 1;
                Ok(JsonValue::Number(*n))
            }
            Token::True => {
                self.pos += 1;
                Ok(JsonValue::Bool(true))
            }
            Token::False => {
                self.pos += 1;
                Ok(JsonValue::Bool(false))
            }
            Token::Null => {
                self.pos += 1;
                Ok(JsonValue::Null)
            }
            e => {
                println!("Error: {:?}", e);
                Err("Invalid JSON structure".to_string())
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // 跳过左 { 字符
        let mut pairs = Vec::new();

        if self.pos < self.tokens.len() && self.tokens[self.pos] == Token::RightBrace {
            self.pos += 1;
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            if self.pos >= self.tokens.len() {
                return Err("Unclosed object".to_string()); // 最后一个字符了还是对象解析，则是没有关闭对象
            }

            // 在 JSON 中 key 必须是一个 String 类型
            let key = match &self.tokens[self.pos] {
                Token::String(s) => s.clone(),
                _ => return Err("Expected string key".to_string()),
            };
            self.pos += 1;

            // key 之后接着应该是一个 : 符号
            if self.pos >= self.tokens.len() || self.tokens[self.pos] != Token::Colon {
                return Err("Expected colon after key".to_string());
            }
            self.pos += 1;

            // 解析出值
            let value = self.parse()?;

            pairs.push((key, value));

            // 解析出值以后到达末尾，则是未关闭的 JSON
            if self.pos >= self.tokens.len() {
                return Err("Unclosed object".to_string());
            }

            // value的下一个字符串必须是：} 或 ,
            match self.tokens[self.pos] {
                Token::RightBrace => {
                    self.pos += 1;
                    break;
                }
                Token::Comma => {
                    self.pos += 1;
                    continue;
                }
                _ => return Err("Expected commna or closing brace".to_string()),
            }
        }

        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        let mut elements = Vec::new();

        if self.pos < self.tokens.len() && self.tokens[self.pos] == Token::RightBracket {
            self.pos += 1;
            return Ok(JsonValue::Array(elements));
        }

        loop {
            if self.pos >= self.tokens.len() {
                return Err("Unclosed array".to_string());
            }

            let value = self.parse()?;
            elements.push(value);

            if self.pos >= self.tokens.len() {
                return Err("Unclosed array".to_string());
            }

            match self.tokens[self.pos] {
                Token::RightBracket => {
                    self.pos += 1;
                    break;
                }
                Token::Comma => {
                    self.pos += 1;
                    continue;
                }
                _ => return Err("Expected comma or closing bracket".to_string()),
            }
        }
        Ok(JsonValue::Array(elements))
    }
}

#[derive(ClapParser, Debug)]
struct Cli {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.file.is_none() {
        eprintln!("file is not provided.");
        std::process::exit(1);
    }

    let json_content = fs::read_to_string(cli.file.unwrap())?;
    let mut lexer = Lexer::new(json_content);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == Token::Eof {
            break;
        }
        if token != Token::Whitespace {
            tokens.push(token);
        }
    }
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(json) => {
            println!("Valid JSON: {:?}", json);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Invalid JSON: {:?}", e);
            std::process::exit(1)
        }
    }
}
