use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, anyhow};
use pest::iterators::Pair;

use super::*;

use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

type NodeIndex = usize;

pub struct Node {
    component: &'static dyn Component,
    state: *mut (),
    outputs: SVec<NodeIndex>
}

impl Node {
    fn new(component: &'static dyn Component) -> &'static mut Self {
        Self { component, state: core::ptr::null_mut(), outputs: Default::default() }.box_and_leak()
    }
}

#[derive(Clone)]
pub struct CNode(NodeIndex, NodeIndex);

/// load a script, build the DAG, initialize the nodes:
pub fn load_script(code: &str, components: &[&'static dyn Component]) -> Result<Vec<&'static mut Node>> {
    enum SymbolValue { CNode(CNode), Function(&'static dyn Component) }

    let mut nodes = vec![];
    let mut arguments = vec![]; // associated arguments for each node. Deallocate after initialization.

    let mut symbol_table: BTreeMap<String, SymbolValue> = BTreeMap::new();
    for &comp in components {
        for fname in comp.functions() {
            symbol_table.insert(fname.to_string(), SymbolValue::Function(comp));
        }
    }

    fn get_lit_value(pair: Pair<Rule>) -> ArgumentValue {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::string => ArgumentValue::String(pair.as_str().to_string()), // TODO: escaping!
            Rule::int => ArgumentValue::Int(pair.as_str().parse().unwrap()),
            _ => unreachable!()
        }
    }

    fn parse_arg(pair: Pair<Rule>) -> Argument {
        let mut pairs = pair.into_inner();
        let first = pairs.next().unwrap();
        match first.as_rule() {
            Rule::lit => Argument("".to_string(), get_lit_value(first)),
            Rule::ident => Argument(first.to_string(), get_lit_value(pairs.next().unwrap())),
            _ => unreachable!()
        }
    }

    fn parse_node(pair: Pair<Rule>) -> (String, Vec<Argument>) {
        assert_eq!(pair.as_rule(), Rule::node);
        let mut pairs = pair.into_inner();
        let ident = pairs.next().unwrap().as_str().to_string();
        let args = pairs.next().map(|x| x.into_inner().map(parse_arg).collect()).unwrap_or_default();
        (ident, args)
    }

    fn eval(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<&'static mut Node>, arguments: &mut Vec<Vec<Argument>>, pair: Pair<Rule>) -> CNode {
        match pair.as_rule() {
            Rule::cnode => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                if let Some(second) = pairs.next() {
                    let (mut forward_node_index, mut backward_node_index) = (0, 0);
                    for ((pair, index), direction) in [first, second].into_iter().zip([&mut forward_node_index, &mut backward_node_index]).zip(["forward", "backward"]) {
                        let (ident, mut args) = parse_node(pair);
                        if let SymbolValue::Function(comp) = &symbol_table[&ident] {
                            *index = nodes.len();
                            nodes.push(Node::new(*comp));
                            args.push(Argument("function_name".into(), ident.into()));
                            args.push(Argument("direction".into(), direction.to_string().into()));
                            arguments.push(args)
                        } else {
                            panic!("the double bang (!!) composition can only be used to combine two function calls.")
                        }
                    }
                    CNode(forward_node_index, backward_node_index)
                } else {
                    let (ident, mut args) = parse_node(first);
                    match &symbol_table[&ident] {
                        SymbolValue::Function(comp) => {
                            args.push(Argument("function_name".into(), ident.into()));

                            let forward_node_index = nodes.len();
                            nodes.push(Node::new(*comp));
                            let mut forward_args = args.clone();
                            forward_args.push(Argument("direction".into(), "forward".to_string().into()));
                            arguments.push(forward_args);

                            let backward_node_index = nodes.len();
                            nodes.push(Node::new(*comp));
                            args.push(Argument("direction".into(), "backward".to_string().into()));
                            arguments.push(args);

                            CNode(forward_node_index, backward_node_index)
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
                    let cnode = eval(symbol_table, nodes, arguments, pair);
                    if let Some(p) = last {
                        nodes[p.0].outputs.push(cnode.0);
                        nodes[cnode.1].outputs.push(p.1);
                    }
                    last = Some(cnode)
                }
                last.unwrap()
            },
            _ => unreachable!(),
        }
    }

    fn walk(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<&'static mut Node>, arguments: &mut Vec<Vec<Argument>>, pair: Pair<Rule>) {
        match pair.as_rule() {
            Rule::EOI | Rule::WHITESPACE => {},
            Rule::stmt => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::assignment => walk(symbol_table, nodes, arguments, pair),
                    Rule::pipe => eval(symbol_table, nodes, arguments, pair).ignore(),
                    _ => unreachable!()
                }
            },
            Rule::assignment => {
                let mut pairs = pair.into_inner();
                let ident = pairs.next().unwrap().as_str().to_string();
                let value = eval(symbol_table, nodes, arguments, pairs.next().unwrap());
                symbol_table.insert(ident, SymbolValue::CNode(value));
            },
            Rule::script => for pair in pair.into_inner() {
                walk(symbol_table, nodes, arguments, pair)
            },
            _ => unreachable!()
        }
    }

    for pair in ScriptParser::parse(Rule::script, code)? {
        walk(&mut symbol_table, &mut nodes, &mut arguments, pair)
    }

    for (node, mut arguments) in nodes.iter_mut().zip(arguments.into_iter()) {
        arguments.push(Argument("n_outputs".into(), (node.outputs.len() as u64).into()));
        node.state = node.component.create(arguments).map_err(|e| anyhow!(e))?.as_ptr();
    }

    Ok(nodes)
}
