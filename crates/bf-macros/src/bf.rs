use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{bracketed, AngleBracketedGenericArguments, GenericArgument, LitInt, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Bracket;

pub struct BfScript {
    // this should come from the actual macro syntax lord.
    // TODO: Later impl: this is being saved for a later version as it increases complexity heavily.
    // cell_type: Type,
    // the amount of negative ptrs that are allowed. Aka offset from 0 idx of mem array to ptr idxs.
    negative_ptrs: usize,
    // the amount of cells we have in memory:
    cells: usize,
    // the actual operators.
    script: Vec<BfOperator>,
}

impl Parse for BfScript {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the config: <cell_type, cells, negative_ptrs>
        // We use AngleBracketedGenericArguments to parse the <...>
        let config: AngleBracketedGenericArguments = input.parse()?;

        let args: Vec<_> = config.args.iter().collect();
        if args.len() != 3 {
            return Err(syn::Error::new(
                config.span(),
                "Expected 3 arguments: <cell_type, cells, negative_ptrs>",
            ));
        }

        // TODO: either get rid of this or actually impl it.
        let _cell_type = match args[0] {
            GenericArgument::Type(ty) => ty.clone(),
            _ => {
                return Err(syn::Error::new(
                    args[0].span(),
                    "Expected a type for argument 1 (cell_type)",
                ))
            }
        };

        let cells_lit = match args[1] {
            GenericArgument::Const(expr) => syn::parse2::<LitInt>(expr.to_token_stream())?,
            _ => {
                return Err(syn::Error::new(
                    args[1].span(),
                    "Expected a const literal for argument 2 (cells)",
                ))
            }
        };
        let cells = cells_lit.base10_parse()?;

        let neg_lit = match args[2] {
            GenericArgument::Const(expr) => syn::parse2::<LitInt>(expr.to_token_stream())?,
            _ => {
                return Err(syn::Error::new(
                    args[2].span(),
                    "Expected a const literal for argument 3 (negative_ptrs)",
                ))
            }
        };


        let negative_ptrs = neg_lit.base10_parse()?;

        let script = if input.peek(LitStr) {
            let script_literal: LitStr = input.parse()?;
            let script_string = script_literal.value();

            // 1. "Re-tokenize" the string
            let token_stream = tokenize_bf_string(&script_string, script_literal.span());

            // 2. Parse that stream using your wrapper
            let ops: BfOperators = syn::parse2(token_stream)?;
            ops.0
        } else {
            parse_all(input)
        };

        Ok(Self {
            negative_ptrs,
            cells,
            script,
        })
    }
}

/// Turns a string like "++[->]" into a TokenStream of `+`, `+`, `[`, etc.
fn tokenize_bf_string(s: &str, span: proc_macro2::Span) -> proc_macro2::TokenStream {
    let mut tokens = proc_macro2::TokenStream::new();
    for c in s.chars() {
        match c {
            // These are valid BF ops, turn them into single tokens
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => {
                let p = proc_macro2::Punct::new(c, proc_macro2::Spacing::Alone);
                // `quote_spanned!` applies the string's span to the new token.
                tokens.extend(quote_spanned! {span=> #p });
            }

            // Ignore whitespace and other chars (which are comments in BF)
            _ => {}
        }
    }
    tokens
}

struct BfOperators(Vec<BfOperator>);

impl Parse for BfOperators {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(BfOperators(parse_all(input)))
    }
}

impl ToTokens for BfScript {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // First we need to destructure self as we cant quasi qoute in self access stuff.
        let cells = self.cells;
        let negative_ptrs = self.negative_ptrs;
        let total_size = cells + negative_ptrs;
        // TODO: hard coded for now when we are ready to change this shit.
        let cell_type = quote! { u8 };

        fn ops_to_token(ops: &Vec<BfOperator>) -> proc_macro2::TokenStream {
            let mut ops_stream = proc_macro2::TokenStream::new();
            for op in ops {
                let op_code = match op {
                    BfOperator::PointerInc => quote! { pointer += 1; },
                    BfOperator::PointerDec => quote! { pointer -= 1; },
                    BfOperator::Add => quote! {
                        let cell = &mut mem[pointer];
                        *cell = cell.wrapping_add(1);
                    },
                    BfOperator::Sub => quote! {
                        let cell = &mut mem[pointer];
                        *cell = cell.wrapping_sub(1);
                    },
                    BfOperator::Output => quote! {
                        result.push(mem[pointer] as char);
                    },
                    // this input we can actually do inputting.
                    // we need to read one char for stdin.
                    // maybe inlining a function for this could be helpful globally or something idk.
                    BfOperator::Input => quote! {
                        let mut buffer = [0_u8]; // buffer to use to read exact.
                        std::io::stdin().read_exact(&mut buffer).expect("error with stdin during bf.");
                        let char_byte = buffer[0];
                    },
                    BfOperator::Loop(l) => {
                        // recursion baby.
                        let loop_body = ops_to_token(&l.inner_operators);
                        // wrap it in a rust while loop.
                        quote! {
                            while mem[pointer] != 0 {
                                #loop_body
                            }
                        }
                    }
                };
                ops_stream.append_all(op_code);
            };
            ops_stream
        }
        let main_body = ops_to_token(&self.script);

        let output = quote! {
            // wrap in a block so that this works properly.
            {
                use std::io::Read;
                let mut mem: ::std::vec::Vec<#cell_type> = ::std::vec![0; #total_size];
                let mut pointer: usize = #negative_ptrs;
                let mut result = ::std::string::String::new();

                #main_body

                result
            }
        };

        // Append the generated Rust block to the output token stream
        tokens.append_all(output);
    }
}

pub struct BfVM {
    bf_script: BfScript,
    state: VmState,
    // No more result, mem, pc, or pointer here!
}

impl BfVM {
    pub fn new(bf_script: BfScript) -> Self {
        let state = VmState {
            result: String::new(),
            // Pre-allocate based on the script's config
            mem: vec![0; bf_script.cells + bf_script.negative_ptrs],
            // The data pointer starts at the offset here makes it easy and allows
            // handling of negatives.
            pointer: bf_script.negative_ptrs,
            program_counter: 0,
        };
        Self { bf_script, state }
    }

    pub fn execute(&mut self) -> String {
        let script = &self.bf_script.script;
        let state = &mut self.state;

        while let Some(instruction) = script.get(state.program_counter) {
            // Pass the instruction to the state's executor
            state.execute_op(instruction);
            state.program_counter += 1;
        }

        // Return the final result
        // We have to use std::mem::take to move the String out
        // of the VmState, which is still borrowed.
        std::mem::take(&mut state.result)
    }
}

struct VmState {
    result: String,
    mem: Vec<u8>,
    program_counter: usize,
    pointer: usize,
}

impl VmState {
    fn current_cell_mut(&mut self) -> &mut u8 {
        if self.pointer >= self.mem.len() {
            self.mem.resize(self.pointer + 1, 0);
        }
        &mut self.mem[self.pointer]
    }

    fn current_cell_value(&self) -> u8 {
        *self.mem.get(self.pointer).unwrap_or(&0)
    }

    fn execute_op(&mut self, instruction: &BfOperator) {
        match instruction {
            BfOperator::PointerInc => {
                self.pointer += 1;
            }
            BfOperator::PointerDec => {
                self.pointer -= 1;
            }
            BfOperator::Add => {
                let cell = self.current_cell_mut();
                *cell = cell.wrapping_add(1);
            }
            BfOperator::Sub => {
                let cell = self.current_cell_mut();
                *cell = cell.wrapping_sub(1);
            }
            BfOperator::Output => {
                let char_val = self.current_cell_value() as char;
                self.result.push(char_val);
            }
            BfOperator::Input => {
                // NOOP
            }
            BfOperator::Loop(l) => {
                // this condition is only checked at the beginning and ending of the loop.
                // if nonzero then loop if its false = 0 then continue.
                while self.current_cell_value() != 0 {
                    for op in &l.inner_operators {
                        // The recursive call is simple and clean
                        self.execute_op(op);
                    }
                }
            }
        }
    }
}

macro_rules! bf_operators {
    ($($variant:ident => [$token:tt]),*) => {
        #[derive(Clone)]
        enum BfOperator {
            $(
                $variant,
            )*
            Loop(Loop)
        }

        impl Parse for BfOperator {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let lookahead = input.lookahead1();
                $(
                if lookahead.peek(Token![$token]) {
                    let _ = input.parse::<Token![$token]>()?;
                    return Ok(BfOperator::$variant);
                }
                )*

                if lookahead.peek(syn::token::Bracket) {
                    return Ok(BfOperator::Loop(input.parse()?));
                }

                Err(lookahead.error())
            }
        }
    };
}

bf_operators!(
    PointerInc      => [>],
    PointerDec      => [<],
    Add             => [+],
    Sub             => [-],
    Output          => [.],
    Input           => [,]
);

#[derive(Clone)]
struct Loop {
    _bracket: Bracket,
    inner_operators: Vec<BfOperator>
}

impl Parse for Loop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner;
        Ok(Loop {
            _bracket: bracketed!(inner in input),
            inner_operators: parse_all(&inner)
        })
    }
}

fn parse_all(input: ParseStream) -> Vec<BfOperator> {
    let mut res = vec![];
    while !input.is_empty() {
        if let Ok(operator) = input.parse() {
            res.push(operator)
        }
        // maybe later add a feature flag or something that spits out
        // if an invalid character was skipped. They are said to be comments but meh.
    }
    res
}
