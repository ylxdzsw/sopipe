use oh_my_rust::*;
use std::collections::BTreeMap;
use std::cell::RefCell;
use anyhow::{Result, anyhow};
use pest::Parser;
use pest::iterators::Pair;

use api::*;

#[derive(pest_derive::Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

// intermediate graph presentation
struct Node {
    spec: &'static dyn ComponentSpec,
    args: Vec<Argument>,
    outputs: Vec<usize>,
    conj: usize,
}

impl Node {
    fn new(spec: &'static dyn ComponentSpec, args: Vec<Argument>) -> RefCell<Self> {
        RefCell::new(Self { spec, args, outputs: Default::default(), conj: Default::default() })
    }
}

#[derive(Clone)]
struct CNode(usize, usize); // CNode stands for "composited node", which includes a forward node and a backward node.

/// load a script, build the DAG, initialize the nodes:
pub(crate) fn load_script(code: &str, specs: &[&'static dyn ComponentSpec]) -> Result<Vec<super::Node>> {
    enum SymbolValue { CNode(CNode), Function(&'static dyn ComponentSpec) }

    let mut nodes = vec![];

    let mut symbol_table: BTreeMap<String, SymbolValue> = BTreeMap::new();
    for &spec in specs {
        for fname in spec.functions() {
            symbol_table.insert(fname.to_string(), SymbolValue::Function(spec));
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
            Rule::ident => Argument(first.as_str().to_string(), pairs.next().map(get_lit_value).unwrap_or(ArgumentValue::None)),
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

    fn eval(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<RefCell<Node>>, pair: Pair<Rule>) -> CNode {
        match pair.as_rule() {
            Rule::cnode => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                if let Some(second) = pairs.next() {
                    let (mut forward_node_index, mut backward_node_index) = (0, 0);
                    for ((pair, index), direction) in [first, second].into_iter().zip([&mut forward_node_index, &mut backward_node_index]).zip(["forward", "backward"]) {
                        let (ident, mut args) = parse_node(pair);
                        if let SymbolValue::Function(spec) = &symbol_table[&ident] {
                            *index = nodes.len();
                            args.push(Argument("function_name".into(), ident.into()));
                            args.push(Argument("direction".into(), direction.to_string().into()));
                            nodes.push(Node::new(*spec, args));
                        } else {
                            panic!("the double bang (!!) composition can only be used to combine two function calls.")
                        }
                    }
                    nodes[forward_node_index].borrow_mut().conj = backward_node_index;
                    nodes[backward_node_index].borrow_mut().conj = forward_node_index;
                    CNode(forward_node_index, backward_node_index)
                } else {
                    let (ident, mut args) = parse_node(first);
                    match &symbol_table[&ident] {
                        SymbolValue::Function(spec) => {
                            args.push(Argument("function_name".into(), ident.into()));

                            let forward_node_index = nodes.len();
                            let mut forward_args = args.clone();
                            forward_args.push(Argument("direction".into(), "forward".to_string().into()));
                            nodes.push(Node::new(*spec, forward_args));

                            let backward_node_index = nodes.len();
                            args.push(Argument("direction".into(), "backward".to_string().into()));
                            nodes.push(Node::new(*spec, args));

                            nodes[forward_node_index].borrow_mut().conj = backward_node_index;
                            nodes[backward_node_index].borrow_mut().conj = forward_node_index;
                            CNode(forward_node_index, backward_node_index)
                        }
                        SymbolValue::CNode(cnode) => {
                            cnode.clone()
                        },
                    }
                }
            },
            Rule::pipe => {
                let mut last: Option<CNode> = None;
                for pair in pair.into_inner() {
                    let cnode = eval(symbol_table, nodes, pair);
                    if let Some(p) = last {
                        nodes[p.0].borrow_mut().outputs.push(cnode.0);
                        nodes[cnode.1].borrow_mut().outputs.push(p.1);
                    }
                    last = Some(cnode)
                }
                last.unwrap()
            },
            _ => unreachable!(),
        }
    }

    fn walk(symbol_table: &mut BTreeMap<String, SymbolValue>, nodes: &mut Vec<RefCell<Node>>, pair: Pair<Rule>) {
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

    nodes.into_iter().map(|node| {
        let Node { spec, mut args, outputs, conj } = node.into_inner();
        args.push(Argument("outputs".into(), outputs.iter().map(|_| "".to_string()).collect()));
        let comp = spec.create(args).map_err(|e| anyhow!(e))?;
        Ok(super::Node { comp: Box::leak(Box::new(comp)), outputs: outputs.leak(), conj })
    }).collect()
}
