use svm::cell::SVMCell;
use svm::cell::Atom::*;
use svm::cell::Inst::*;
use svm::cell::SVMCell::*;
use svm::slist::{List,Stack};
use svm::slist::List::{Cons,Nil};

use self::ExprNode::*;
use self::NumNode::*;
use super::ForkTable;

use std::fmt;
use std::iter::FromIterator;
use std::convert::Into;
use std::cmp::max;
use std::hash::Hash;

#[cfg(test)]
mod tests;

/// The symbol table for bound names is represented as a
/// `ForkTable` mapping `&str` (names) to `(uint,uint)` tuples,
/// representing the location in the `$e` stack storing the value
/// bound to that name.
#[stable(feature = "forktable", since = "0.0.6")]
pub type SymTable<'a>   = ForkTable<'a, &'a str, usize>;

/// A `CompileResult` is either `Ok(SVMCell)` or `Err(&str)`
#[stable(feature = "compile", since = "0.0.3")]
pub type CompileResult  = Result<Vec<SVMCell>, String>;

static INDENT: &'static str = "\t";

/// Trait for a symbol table
pub trait Scope<K> where K: Eq + Hash {
    /// Bind a name to a scope.
    ///
    /// Returnsthe indices for that name in the SVM environment.
    fn bind(&mut self, name: K)  -> (usize,usize);
    /// Look up a name against a scope.
    ///
    /// Returns the indices for that name in the SVM environment,
    /// or None if that name is unbound.
    fn lookup(&self, name: &K)   -> Option<(usize,usize)>;
}

/// Trait for AST nodes.
#[stable(feature = "ast", since = "0.0.2")]
pub trait ASTNode {
    /// Compile this node to a list of SVM expressions
    #[unstable(feature="compile")]
    fn compile<'a>(&'a self,
                   state: &'a SymTable<'a>
                   )                    -> CompileResult;

    /// Pretty-print this node
    #[stable(feature = "ast", since = "0.0.2")]
    fn prettyprint(&self)               -> String { self.print_level(0) }

    /// Pretty-print this node at the desired indent level
    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String;
}

impl fmt::Debug for ASTNode {
    #[stable(feature = "ast", since = "0.0.4")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

/// Expression
///
/// All Seax Scheme expressions are one of the following
///
///  + Nested S-Expressions
///  + Identifiers
///  + Lists
///  + Numbers
///     - signed int
///     - unsigned int
///     - floating-point
///  + Characters
///  + Strings
///
///  TODO: implement the entire Scheme 'numeric tower'
///  TODO: macros should happen
///  TODO: figure out quasiquote somehow.
#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub enum ExprNode {
    #[stable(feature = "ast", since = "0.0.2")]
    Root(RootNode),
    #[stable(feature = "ast", since = "0.0.2")]
    SExpr(SExprNode),
    #[stable(feature = "ast", since = "0.0.2")]
    Name(NameNode),
    #[stable(feature = "ast", since = "0.0.2")]
    ListConst(ListNode),
    #[stable(feature = "ast", since = "0.0.2")]
    NumConst(NumNode),
    #[stable(feature = "ast", since = "0.0.2")]
    BoolConst(BoolNode),
    #[stable(feature = "ast", since = "0.0.2")]
    StringConst(StringNode),
    #[stable(feature = "ast", since = "0.0.2")]
    CharConst(CharNode),
}

impl ASTNode for ExprNode {

    #[stable(feature = "compile", since = "0.0.3")]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        match *self {
            //  TODO: should some of these nodes cause a state fork?
            Root(ref node)          => node.compile(state),
            SExpr(ref node)         => node.compile(state),
            Name(ref node)          => node.compile(state),
            ListConst(ref node)     => node.compile(state),
            NumConst(ref node)      => node.compile(state),
            BoolConst(ref node)     => node.compile(state),
            CharConst(ref node)     => node.compile(state),
            StringConst(ref node)   => node.compile(state)
        }
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        match *self {
            Root(ref node)          => node.print_level(level),
            SExpr(ref node)         => node.print_level(level),
            Name(ref node)          => node.print_level(level),
            ListConst(ref node)     => node.print_level(level),
            NumConst(ref node)      => node.print_level(level),
            BoolConst(ref node)     => node.print_level(level),
            CharConst(ref node)     => node.print_level(level),
            StringConst(ref node)   => node.print_level(level)
        }
    }
}

impl fmt::Debug for ExprNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub enum NumNode {
    #[stable(feature = "ast", since = "0.0.2")]
    IntConst(IntNode),
    #[stable(feature = "ast", since = "0.0.2")]
    UIntConst(UIntNode),
    #[stable(feature = "ast", since = "0.0.2")]
    FloatConst(FloatNode)
}

impl fmt::Debug for NumNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

/// AST node for the root of a program's AST
#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct RootNode { pub exprs: Vec<ExprNode> }

impl fmt::Debug for RootNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

impl ASTNode for RootNode {
    #[unstable(feature="compile")]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        Err("UNINPLEMENTED".to_string())
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        self.exprs
            .iter()
            .fold(
                String::new(),
                |mut s, i| {
                    s.push_str(i.print_level(level + 1).as_ref());
                    s
                })
    }

}

/// AST node for an S-expression.
///
/// This includes function application, assignment,
/// function definition, et cetera...Scheme is not a complexl anguage.
#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct SExprNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub operator: NameNode,
    #[stable(feature = "ast", since = "0.0.2")]
    pub operands: Vec<ExprNode>,
}

impl ASTNode for SExprNode {
    #[unstable(feature="compile")]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        match self.operator.name.as_ref() {
            "if" => match self.operands.as_slice() {
                [ref condition,ref true_case,ref false_case] => {
                    let mut result = Vec::new();

                    result.push_all(&try!(condition.compile(state)));
                    result.push(InstCell(SEL));

                    let mut false_code = try!(false_case.compile(state));
                    false_code.push(InstCell(JOIN));

                    let mut true_code = try!(true_case.compile(state));
                    true_code.push(InstCell(JOIN));

                    result.push(ListCell(box List::from_iter(true_code)));
                    result.push(ListCell(box List::from_iter(false_code)));

                    Ok(result)
                },
                _ => Err("[error]: malformed if expression".to_string())
            },
            "lambda" => match self.operands.as_slice() {
                [SExpr(SExprNode{
                            operator: ref param_a,
                            operands: ref param_bs}), SExpr(ref body)] => {
                    let mut sym = state.fork();
                    let _ = sym.bind(param_a.name.as_ref());
                    for b in param_bs {
                        if let &Name(ref node) = b {
                            let _ = sym.bind(node.name.as_ref());
                        } // todo: make errors otherwise
                    }
                    let mut result = Vec::new();
                    Ok(result)
                },
                _ => Err("[error]: malformed lambda expression".to_string())
            },
            _    => {
                let ref op = self.operator;
                let mut result = Vec::new();
                match self.operands {
                    ref other if other.len() == 1 => {
                        result.push_all(&try!(other[0].compile(state)));
                        result.push_all(&try!(op.compile(state)));
                    },
                    _       => {
                        let mut it = self.operands.iter().rev();
                        // TODO: can thsi be represented with a reduce/fold?
                        result.push_all(&try!(
                            it.next().unwrap().compile(state)));
                        for ref operand in it {
                            result.push_all(&try!(operand.compile(state)));
                            result.push_all(&try!(op.compile(state)));
                        }
                    }
                }

                Ok(result)

            }
        }
    }

    #[stable(feature = "ast", since = "0.0.6")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level { tab.push_str(INDENT); };

        let mut result = String::new();
        result.push_str(tab.as_ref());
        result.push_str("S-Expression:\n");
        tab.push_str(INDENT);

        // op
        result.push_str(tab.as_ref());
        result.push_str("Operator:\n");
        result.push_str(self.operator.print_level(level + 2).as_ref());

        for ref operand in self.operands.iter() {
            result.push_str(tab.as_ref());
            result.push_str("Operand: \n");
            result.push_str(operand.print_level(level + 2).as_ref());
        };
        result
    }

}
#[stable(feature = "ast", since = "0.0.4")]
impl fmt::Debug for SExprNode {
    #[stable(feature = "ast", since = "0.0.2")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

/// AST node for a list literal
#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct ListNode { pub elements: Vec<ExprNode> }
#[stable(feature = "ast", since = "0.0.4")]
impl fmt::Debug for ListNode {
    #[stable(feature = "ast", since = "0.0.4")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

impl ASTNode for ListNode {
    #[unstable(feature="compile")]
    fn compile<'a>(&'a self, state: &SymTable<'a>) -> CompileResult {
        Err("UNINPLEMENTED".to_string())
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT); };

        let mut result = String::new();
        result.push_str("List:\n");
        tab.push_str(INDENT);

        for elem in self.elements.iter() {
            result.push_str(tab.as_ref());
            result.push_str(elem.print_level(level + 1).as_ref());
            result.push('\n');
        };
        result
    }

}

/// AST node for an identifier
#[derive(Clone, PartialEq)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct NameNode { pub name: String }

impl NameNode {
    /// Returns true if this is a keyword
    #[stable(feature = "ast", since = "0.0.3")]
    fn is_kw(&self) -> bool {
        match self.name.as_ref() {
            "access" | "define-syntax" | "macro"  | "and"  | "delay"
            | "make-environment" | "begin"  | "do"| "named-lambda"
            | "bkpt" | "fluid-let" | "or" | "case" | "if" | "quasiquote"
            | "cond" | "in-package" | "quote" | "cons-stream" | "lambda"
            | "scode-quote" | "declare" | "let" | "sequence" | "default-object?"
            | "let*" | "set!" | "define" | "let-syntax" | "the-environment"
            | "define-integrable" | "letrec" | "unassigned?" | "define-macro"
            | "local-declare" | "using-syntax" | "define-structure" | "car"
            | "cdr" | "cons" | "nil" | "nil?" | "atom?" => true,
            _ => false
        }
    }
    /// Returns true if this is an arithmetic operator
    #[stable(feature = "ast", since = "0.0.3")]
    fn is_arith(&self) -> bool {
      match self.name.as_ref() {
         "+" | "-" | "*" | "/" | "%" => true,
         _ => false
      }
   }
    /// Returns true if this is a comparison operator
    #[stable(feature = "ast", since = "0.0.3")]
    fn is_cmp(&self) -> bool {
      match self.name.as_ref() {
         "=" | "!=" | ">" | "<" | ">=" | "<=" => true,
         _ => false
      }
   }

   #[stable(feature = "ast", since = "0.0.4")]
   pub fn new(name: String) -> Self { NameNode {name: name} }
}
#[stable(feature = "ast", since = "0.0.4")]
impl fmt::Debug for NameNode {
    #[stable(feature = "ast", since = "0.0.4")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prettyprint())
    }
}

impl ASTNode for NameNode {
    #[unstable(feature="compile")]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        match self.name.as_ref() {
            "cons"  => Ok(vec![InstCell(CONS)]),
            "car"   => Ok(vec![InstCell(CAR)]),
            "cdr"   => Ok(vec![InstCell(CDR)]),
            "nil"   => Ok(vec![InstCell(NIL)]),
            "nil?"  => Ok(vec![InstCell(NULL)]),
            "atom?" => Ok(vec![InstCell(ATOM)]),
            "+"     => Ok(vec![InstCell(ADD)]),
            "-"     => Ok(vec![InstCell(SUB)]),
            "*"     => Ok(vec![InstCell(MUL)]),
            "/"     => Ok(vec![InstCell(DIV)]),
            "%"     => Ok(vec![InstCell(MOD)]),
            "="     => Ok(vec![InstCell(EQ)]),
            ">"     => Ok(vec![InstCell(GT)]),
            ">="    => Ok(vec![InstCell(GTE)]),
            "<"     => Ok(vec![InstCell(LT)]),
            "<="    => Ok(vec![InstCell(LTE)]),
            ref name => match state.lookup(&name) {
                Some((x,y)) => unimplemented!(),
                None        => Err(format!(
                    "[error] Unknown identifier `{}`", name))
            }
        }
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT)};

        let mut result = String::new();
        result.push_str(tab.as_ref());
        result.push_str("Name: ");
        result.push_str(self.name.as_ref());
        result.push_str("\n");

        result
    }

}

/// AST node for an integer constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct IntNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub value: isize
}

impl ASTNode for NumNode {
    #[stable(feature="compile",since="0.0.3")]
    #[allow(unused_variables)]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
       match *self {
            NumNode::UIntConst(ref node)    => Ok(
                    vec![InstCell(LDC),AtomCell(UInt(node.value))]
                ),
            NumNode::IntConst(ref node)     => Ok(
                    vec![InstCell(LDC),AtomCell(SInt(node.value))]
                ),
            NumNode::FloatConst(ref node)   => Ok(
                    vec![InstCell(LDC),AtomCell(Float(node.value))]
                )
       }
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT);};

        let mut result = String::new();

        result.push_str(tab.as_ref());
        result.push_str("Number: ");

        match *self {
            NumNode::UIntConst(ref node) => {
                result.push_str(format!("{}u", node.value).as_ref());
                result.push_str("\n");
            },
            NumNode::IntConst(ref node) => {
                result.push_str(format!("{}", node.value).as_ref());
                result.push_str("\n");
            },
            NumNode::FloatConst(ref node) => {
                result.push_str(format!("{}f", node.value).as_ref());
                result.push_str("\n");
            }
        }
        result
    }
}

/// AST node for an unsigned integer constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct UIntNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub value: usize
}

/// AST node for a floating-point constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct FloatNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub value: f64
}

/// AST node for a boolean constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct BoolNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub value: bool
}

impl ASTNode for BoolNode {
    #[stable(feature="compile", since="0.0.6")]
    #[allow(unused_variables)]
    fn compile<'a>(&'a self,state:  &'a SymTable)    -> CompileResult {
        match self.value {
            true    => Ok(vec![InstCell(LDC), AtomCell(SInt(1))]),
            false   => Ok(vec![InstCell(NIL)])
        }
    }

    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT);};

        let mut result = String::new();

        result.push_str(tab.as_ref());
        result.push_str("Boolean: ");
        result.push_str(format!("{}", self.value).as_ref());
        result.push_str("\n");
        result
    }
}


/// AST node for a character constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct CharNode {
    #[stable(feature = "ast", since = "0.0.2")]
    pub value: char
}

impl ASTNode for CharNode {
    #[stable(feature="compile", since="0.0.7")]
    #[allow(unused_variables)]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        Ok(vec![AtomCell(Char(self.value))])
    }
    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT);};

        let mut result = String::new();

        result.push_str("Character: \'");
        result.push(self.value);
        result.push_str("\'\n");
        result
    }
}


/// AST node for a  string constant
#[derive(Clone, PartialEq,Debug)]
#[stable(feature = "ast", since = "0.0.2")]
pub struct StringNode { pub value: String }

impl ASTNode for StringNode {
    #[unstable(feature="compile")]
    #[allow(unused_variables)]
    fn compile<'a>(&'a self, state: &'a SymTable<'a>) -> CompileResult {
        let chars: Vec<u8> = self.value.clone().into();
        Ok(vec![
            ListCell(box List::from_iter(
                chars.into_iter().map(|c| AtomCell(Char(c as char)))
                )) ])
    }
    #[stable(feature = "ast", since = "0.0.2")]
    fn print_level(&self, level: usize) -> String {
        let mut tab = String::new();
        for _ in 0 .. level {tab.push_str(INDENT);};

        let mut result = String::new();

        result.push_str("String: \"");
        result.push_str(self.value.as_ref());
        result.push_str("\"\n");
        result
    }
}

