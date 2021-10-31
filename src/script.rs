use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::Result;
use pest::iterators::Pair;

use super::*;

use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

pub struct Node {
    component: &'static dyn Component,
    args: Vec<Argument>,
    outputs: SVec<Rc<RefCell<Node>>>
}

#[derive(Clone)]
pub struct CNode(Rc<RefCell<Node>>, Rc<RefCell<Node>>);

fn load_script(code: &str, components: &[&'static dyn Component]) -> Result<Vec<Rc<RefCell<Node>>>> {
    enum SymbolValue { CNode(CNode), Function(&'static dyn Component) }

    let mut nodes = vec![];

    let mut symbol_table: BTreeMap<String, SymbolValue> = BTreeMap::new(); // TODO: use smartstring for identifiers?


    fn get_lit_value(pair: &Pair<Rule>) -> ArgumentValue {
        match pair.as_rule() {
            Rule::string => ArgumentValue::String(pair.as_str().to_string()),
            Rule::int => ArgumentValue::Int(pair.as_str().parse().unwrap()),
            _ => unreachable!()
        }
    }

    fn parse_arg(pair: Pair<Rule>) -> Argument {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        match first.as_rule() {
            Rule::lit => Argument("".to_string(), get_lit_value(&first)),
            Rule::ident => Argument(first.to_string(), get_lit_value(&pairs.next().unwrap())),
            _ => unreachable!()
        }
    }

    fn parse_node(pair: Pair<Rule>) -> (String, Vec<Argument>) {
        assert_eq!(pair.as_rule(), Rule::node);
        let mut pairs = pair.into_inner();
        let ident = pairs.next().unwrap().as_str().to_string();
        let args = pairs.next().unwrap().into_inner().map(parse_arg).collect();
        (ident, args)
    }

    fn eval(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<Rc<RefCell<Node>>>, pair: Pair<Rule>) -> CNode {
        match pair.as_rule() {
            Rule::cnode => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                if let Some(second) = pairs.next() {
                    todo!()
                } else {
                    let (ident, args) = parse_node(first);
                    match &symbol_table[&ident] {
                        SymbolValue::Function(comp) => {
                            let forward_node = Rc::new(RefCell::new(Node { component: *comp, args: args.clone(), outputs: Default::default() }));
                            let backward_node = Rc::new(RefCell::new(Node { component: *comp, args, outputs: Default::default() }));
                            nodes.push(forward_node.clone());
                            nodes.push(backward_node.clone());
                            CNode(forward_node, backward_node)
                        }
                        SymbolValue::CNode(cnode) => {
                            cnode.clone() // it is two `Rc`s
                        },
                    }
                }
            },
            Rule::pipe => {
                let mut last: Option<CNode> = None;
                for pair in pair.into_inner() {
                    let cnode = eval(symbol_table, nodes, pair);
                    if let Some(p) = last {
                        p.0.borrow_mut().outputs.push(cnode.0.clone());
                        cnode.1.borrow_mut().outputs.push(p.1.clone());
                    }
                    last = Some(cnode)
                }
                last.unwrap()
            },
            _ => unreachable!(),
        }
    }

    fn walk(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<Rc<RefCell<Node>>>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::EOI | Rule::WHITESPACE => {},
            Rule::stmt => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::assignment => walk(symbol_table, nodes, pair),
                    Rule::pipe => eval(symbol_table, nodes, pair).ignore(),
                    _ => unreachable!()
                }
            },
            Rule::assignment => {
                let mut pairs = pair.into_inner();
                let ident = pairs.next().unwrap().as_str().to_string();
                let value = eval(symbol_table, nodes, pairs.next().unwrap());
                symbol_table.insert(ident, SymbolValue::CNode(value));
            },
            Rule::script => for pair in pair.into_inner() {
                walk(symbol_table, nodes, pair)
            },
            _ => unreachable!()
        }
    }

    for pair in ScriptParser::parse(Rule::script, code)? {
        walk(&mut symbol_table, &mut nodes, pair)
    }

    for node in nodes.iter() {
        println!("{}", node.borrow().outputs.len())
    }

    Ok(vec![])

}

// additional functions
// env(): get env
// compose(forward, backward): compose two componenets, for each directions
//   explore if we can implement a custom token and syntax for it
