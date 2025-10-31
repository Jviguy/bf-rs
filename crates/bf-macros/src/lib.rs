mod bf;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input};
use quote::quote;
use crate::bf::BfVM;
/*
ok so brainfuck is defined as this:
> 	Move the pointer to the right
< 	Move the pointer to the left
+ 	Increment the memory cell at the pointer
- 	Decrement the memory cell at the pointer
. 	Output the character signified by the cell at the pointer
, 	Input a character and store it in the cell at the pointer
[ 	Jump past the matching ] if the cell at the pointer is 0
] 	Jump back to the matching [ if the cell at the pointer is nonzero.
Only weird things are memory conventions. Brainfuck is loosely defined like a turing machine.
Ptr points to cells, cells are atleast 8 bits but can be more. So need to allow that to be defined.
Also ptr is assumed to never go negative but they do say we should allow for it.
Also on that how many cells there are is a choice to the user but
the original default is around 30000.
So we need to provide syntax for the code itself, cell type, cell amount, cell offset.
the offset is what allows us to do the whole thing of like where ptr 0 is in the cell array.

My original idea with this is to translate the bf into rust code. But now that im thinking about it.
Could run it at compile time and then throw out the tokens of the result haha.

ok two separate macros that have the same parsing do different end goals.
that makes sense to me.

so the compile time one is actually easier i think. Because quote! generating ToTokens
for an Add variant of the enum is harder to generate as it needs access to the array to mutablly add
to it. hmmmmm.

Start with compile time one.
 */

/// This runs the provided bf script at runtime except for input tokens.
/// As we can't access a stdin during compilation time of your project.
/// proc_macros oh yeah.
#[proc_macro]
pub fn const_bf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as bf::BfScript);
    let output = BfVM::new(input).execute();
    let tokens = quote! {
        #output
    };

    tokens.into()
}

#[proc_macro]
pub fn bf(input: TokenStream) -> TokenStream {
    // 1. Parse the input into your BfScript AST
    let script = parse_macro_input!(input as bf::BfScript);

    // 2. Call your new ToTokens implementation
    // `quote!` will see `script` (a BfScript) and call `script.to_tokens(...)`
    let expanded = quote! {
        #script
    };

    // 3. Return the generated Rust code
    expanded.into()
}