use std::collections::HashMap;
use std::rc::Rc;

use std::cell::{
    RefCell,
    Cell
};

use html5ever::tendril::*;
use html5ever::tokenizer::{
    BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    ParseError, CharacterTokens, EndTag, NullCharacterToken, StartTag, TagToken
};

#[derive(Clone)]
struct TokenPrinter {
    in_char_run: Cell<bool>,
    current_tag: Rc<RefCell<String>>,
    tag_stack: Rc<RefCell<Vec<String>>>,
    word_tag_counts: Rc<RefCell<HashMap<String, HashMap<String, usize>>>>,
}

impl TokenPrinter {
    fn is_char(&self, is_char: bool) {
        // match (self.in_char_run.get(), is_char) {
        //     (false, true) => print!("CHAR : \""),
        //     (true, false) => println!("\""),
        //     _ => (),
        // }
        self.in_char_run.set(is_char);
    }

    fn do_char(&self, c: char) {
        self.is_char(true);
        print!("{}", c.escape_default().collect::<String>());
    }
    
    fn count_words(&self, text: &str) {
        let mut word_tags = self.word_tag_counts.borrow_mut();
        let current_tag = self.current_tag.borrow().clone();
        
        // Split the text into words and count them
        for word in text.split_whitespace() {
            // Normalize the word (lowercase and remove punctuation)
            let clean_word = word.chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'')
                .collect::<String>()
                .to_lowercase();
                
            if !clean_word.is_empty() {
                // Get or create the tag -> count map for this word
                let tag_counts = word_tags.entry(clean_word).or_insert_with(HashMap::new);
                // Increment the count for the current tag
                *tag_counts.entry(current_tag.clone()).or_insert(0) += 1;
            }
        }
    }
    
    fn get_word_tag_counts(&self) -> HashMap<String, HashMap<String, usize>> {
        self.word_tag_counts.borrow().clone()
    }
    
}

impl TokenSink for TokenPrinter {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match token {
            CharacterTokens(b) => {
                let text = b.to_string();
                self.count_words(&text);
                
                for c in b.chars() {
                    self.do_char(c);
                }
            },
            NullCharacterToken => self.do_char('\0'),
            TagToken(tag) => {
                self.is_char(false);
                
                // Handle tag stack and current tag
                match tag.kind {
                    StartTag => {
                        // Save current tag to stack before changing
                        let current = self.current_tag.borrow().clone();
                        self.tag_stack.borrow_mut().push(current);
                        
                        // Update current tag
                        *self.current_tag.borrow_mut() = tag.name.to_string();
                        // print!("TAG  : <\x1b[32m{}\x1b[0m", tag.name);
                    },
                    EndTag => {
                        // Pop from stack to return to parent tag
                        if let Some(parent_tag) = self.tag_stack.borrow_mut().pop() {
                            *self.current_tag.borrow_mut() = parent_tag;
                        }
                        // print!("TAG  : <\x1b[31m/{}\x1b[0m", tag.name);
                    },
                }
                
                // Print attributes
                for attr in tag.attrs.iter() {
                    // print!(
                    //     " \x1b[36m{}\x1b[0m='\x1b[34m{}\x1b[0m'",
                    //     attr.name.local, attr.value
                    // );
                }
                if tag.self_closing {
                    // print!(" \x1b[31m/\x1b[0m");
                }
                // println!(">");
            },
            ParseError(err) => {
                self.is_char(false);
                // println!("ERROR: {err}");
            },
            _ => {
                self.is_char(false);
                // println!("OTHER: {token:?}");
            },
        }
        TokenSinkResult::Continue
    }
}

fn main() {
    let sink = TokenPrinter {
        in_char_run: Cell::new(false),
        current_tag: Rc::new(RefCell::new("root".to_string())),
        tag_stack: Rc::new(RefCell::new(Vec::new())),
        word_tag_counts: Rc::new(RefCell::new(HashMap::new())),
    };

    // HTML content to tokenize
    let my_html = r#"
    <html>
    <head>
        <title>My Variable HTML</title>
    </head>
    <body> yeee
        <h1>Hello from a variable!</h1>
        <p>This <span>yeee</span> is a sample paragraph with some repeated words. The words in this paragraph will be counted.</p>
        <div>Sample text with sample words to show how repetition is counted across different tags.</div> yeee
    </body>
    </html>
    "#;

   

    // Create a ByteTendril from our HTML string
    let mut chunk = ByteTendril::new();
    chunk.push_slice(my_html.as_bytes());
    
    let input = BufferQueue::default();
    input.push_back(chunk.try_reinterpret().unwrap());

    let tok = Tokenizer::new(
        sink.clone(),
        TokenizerOpts {
            profile: true,
            ..Default::default()
        },
    );
    let _ = tok.feed(&input);

    assert!(input.is_empty());
    tok.end();
    tok.sink.is_char(false);
    
    // Print word count statistics
    
    // Get the HashMap if needed for further processing
    let word_tag_counts = sink.get_word_tag_counts();

    println!("\n===== FINAL WORD TAG COUNTS =====");
    println!("{:?}", word_tag_counts);
}